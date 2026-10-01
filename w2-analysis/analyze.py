#!/usr/bin/env python3
"""Разбор Solana-транзакции по signature (только чтение).

Использование:
  python3 analyze.py <signature> [mainnet|devnet]
  python3 analyze.py --file tx3.json [mainnet|devnet]   # разбор сохранённого ответа getTransaction
"""
import base64, hashlib, json, math, struct, sys
from collections import defaultdict
from datetime import datetime, timezone

from rpc import call, get_tx

LAMPORTS = 1_000_000_000

# Program ID -> человекочитаемое имя (источники: документация Solana, jup-ag/jupiter-cpi, raydium-io/raydium-sdk-V2)
PROGRAMS = {
    '11111111111111111111111111111111': 'System Program',
    'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA': 'SPL Token Program',
    'TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb': 'Token-2022 Program',
    'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL': 'Associated Token Account Program',
    'ComputeBudget111111111111111111111111111111': 'Compute Budget Program',
    'MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr': 'Memo Program v2',
    'Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo': 'Memo Program v1',
    'JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4': 'Jupiter Aggregator v6',
    'CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK': 'Raydium CLMM',
    'LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj': 'Raydium LaunchLab (Launchpad)',
}

KNOWN_MINTS = {
    'So11111111111111111111111111111111111111112': 'wSOL',
    'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v': 'USDC',
    'Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB': 'USDT',
}

# Кандидаты в имена Anchor-инструкций. Имя показывается, только если совпал хэш sha256("global:<имя>")[:8].
ANCHOR_CANDIDATES = {
    'JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4': [
        'route', 'shared_accounts_route', 'exact_out_route', 'shared_accounts_exact_out_route',
        'route_with_token_ledger', 'shared_accounts_route_with_token_ledger', 'route_v2',
        'shared_accounts_route_v2', 'exact_out_route_v2', 'shared_accounts_exact_out_route_v2',
        'claim', 'claim_token', 'create_token_ledger', 'set_token_ledger', 'close_token'],
    'CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK': ['swap', 'swap_v2', 'swap_router_base_in'],
    'LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj': [
        'buy_exact_in', 'buy_exact_out', 'sell_exact_in', 'sell_exact_out'],
}
# EVENT_IX_TAG в Anchor — u64 0x1d9acb512ea545e4 (= sha256("anchor:event")[:8]), в данных лежит little-endian
ANCHOR_EVENT_TAG = hashlib.sha256(b'anchor:event').digest()[:8][::-1]
ANCHOR_EVENTS = ['SwapEvent', 'SwapsEvent', 'FeeEvent', 'TradeEvent', 'SwapEventV2']

B58 = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'


def b58decode(s):
    n = 0
    for c in s:
        n = n * 58 + B58.index(c)
    raw = n.to_bytes((n.bit_length() + 7) // 8, 'big') if n else b''
    return b'\0' * (len(s) - len(s.lstrip('1'))) + raw


def disc(prefix, name):
    return hashlib.sha256(f'{prefix}:{name}'.encode()).digest()[:8]


def short(a):
    return a if len(a) <= 12 else f'{a[:4]}…{a[-4:]}'


def pname(pid):
    return PROGRAMS.get(pid, f'неизвестная программа {pid}')


def sol(lamports):
    return f'{lamports / LAMPORTS:.9f}'.rstrip('0').rstrip('.')


def decode_unparsed(ix):
    """Тип инструкции для программ, которые RPC не распарсил (data в base58)."""
    pid, data = ix['programId'], b58decode(ix.get('data', ''))
    if pid == 'ComputeBudget111111111111111111111111111111' and data:
        tag = data[0]
        if tag == 2:
            return 'SetComputeUnitLimit', {'units': struct.unpack_from('<I', data, 1)[0]}
        if tag == 3:
            return 'SetComputeUnitPrice', {'microLamports': struct.unpack_from('<Q', data, 1)[0]}
        if tag == 1:
            return 'RequestHeapFrame', {'bytes': struct.unpack_from('<I', data, 1)[0]}
        if tag == 4:
            return 'SetLoadedAccountsDataSizeLimit', {'bytes': struct.unpack_from('<I', data, 1)[0]}
        return f'неизвестный тег {tag}', {}
    if data[:8] == ANCHOR_EVENT_TAG:
        for ev in ANCHOR_EVENTS:
            if data[8:16] == disc('event', ev):
                return f'emit event {ev} (self-CPI)', {}
        return 'emit Anchor event (self-CPI, имя не определено)', {}
    for name in ANCHOR_CANDIDATES.get(pid, []):
        if data[:8] == disc('global', name):
            return name, {}
    return f'не расшифровано (discriminator {data[:8].hex()})' if data else 'не расшифровано', {}


def ix_view(ix):
    """(тип, детали, список аккаунтов) для одной инструкции."""
    if isinstance(ix.get('parsed'), dict):
        info = ix['parsed'].get('info', {})
        accts = [(k, v) for k, v in info.items() if isinstance(v, str) and 32 <= len(v) <= 44]
        details = {k: v for k, v in info.items() if not (isinstance(v, str) and 32 <= len(v) <= 44)}
        for k, v in list(details.items()):
            if isinstance(v, dict) and 'uiAmountString' in v:
                details[k] = f"{v['uiAmountString']} (raw {v['amount']}, decimals {v['decimals']})"
        return ix['parsed'].get('type', '?'), details, accts
    if isinstance(ix.get('parsed'), str):  # Memo
        return 'memo', {'text': ix['parsed']}, [(f'#{i}', a) for i, a in enumerate(ix.get('accounts', []))]
    t, details = decode_unparsed(ix)
    return t, details, [(f'#{i}', a) for i, a in enumerate(ix.get('accounts', []))]


MINT_FEES = {}


def mint_label(mint, network, cache={}):
    if mint in KNOWN_MINTS:
        return KNOWN_MINTS[mint]
    if mint not in cache:
        cache[mint] = short(mint)
        try:  # Token-2022 extensions (у классических SPL-токенов метаданные в Metaplex, их не тянем)
            v = call(network, 'getAccountInfo', [mint, {'encoding': 'jsonParsed'}])['result']['value']
            for ext in v['data']['parsed']['info'].get('extensions', []):
                if ext.get('extension') == 'tokenMetadata':
                    cache[mint] = f"{ext['state']['symbol']} ({short(mint)})"
                if ext.get('extension') == 'transferFeeConfig':
                    bps = ext['state']['newerTransferFee']['transferFeeBasisPoints']
                    if bps:
                        MINT_FEES[mint] = bps
        except Exception:
            pass
    return cache[mint]


def analyze(resp, network, data=None):
    """Текстовый разбор; если передан dict data — заполняет его структурированными данными (для отчёта)."""
    data = {} if data is None else data
    data.update(accounts=[], ix=[], sol=[], tokens=[])
    r = resp['result']
    meta, tx = r['meta'], r['transaction']
    msg = tx['message']
    keys = msg['accountKeys']
    addrs = [k['pubkey'] for k in keys]
    out = []
    p = out.append

    p(f"Signature: {tx['signatures'][0]}")
    p(f"Сеть: {network}   Версия: {r.get('version')}   Статус: {'успех' if meta['err'] is None else meta['err']}")
    bt = datetime.fromtimestamp(r['blockTime'], timezone.utc).strftime('%Y-%m-%d %H:%M:%S UTC')
    p(f"Slot: {r['slot']}   Время: {bt}")
    signers = [k['pubkey'] for k in keys if k['signer']]
    p(f"Fee payer: {addrs[0]}")
    p(f"Signers ({len(signers)}): {', '.join(signers)}")
    data.update(signature=tx['signatures'][0], slot=r['slot'], time=bt, version=r.get('version'),
                fee_payer=addrs[0], signers=signers)

    # --- роли аккаунтов ---
    invoked = {ix['programId'] for ix in msg['instructions']}
    for g in meta.get('innerInstructions') or []:
        invoked |= {ix['programId'] for ix in g['instructions']}
    tok = {}
    for b in (meta.get('preTokenBalances') or []) + (meta.get('postTokenBalances') or []):
        tok[b['accountIndex']] = b
    mints = {b['mint'] for b in tok.values()}
    owners = {b.get('owner') for b in tok.values()}
    hints = defaultdict(set)
    all_ix = list(msg['instructions']) + [i for g in meta.get('innerInstructions') or [] for i in g['instructions']]
    for ix in all_ix:
        if isinstance(ix.get('parsed'), dict):
            t = ix['parsed'].get('type')
            for k, v in ix['parsed'].get('info', {}).items():
                if isinstance(v, str) and v in addrs:
                    hints[v].add(f'{k} в {t}')

    p(f"\nAccounts ({len(addrs)}):")
    for i, k in enumerate(keys):
        a = k['pubkey']
        flags = ('signer' if k['signer'] else '') + (',' if k['signer'] else '') + ('writable' if k['writable'] else 'readonly')
        role = []
        if i == 0:
            role.append('fee payer')
        if a in invoked:
            role.append(f'программа: {pname(a)}')
        elif a in PROGRAMS:
            role.append(f'программа (передана как аккаунт): {pname(a)}')
        if a.startswith('Sysvar'):
            role.append('sysvar')
        if i in tok:
            role.append(f"токен-аккаунт {mint_label(tok[i]['mint'], network)}, владелец {short(tok[i].get('owner', '?'))}")
        if a in mints:
            role.append(f'mint токена {mint_label(a, network)}')
        if a in owners and i not in tok:
            role.append('владелец токен-аккаунтов')
        if {'newAccount в createAccount', 'account в closeAccount'} <= hints[a]:
            role.append('временный токен-аккаунт: создан и закрыт внутри этой транзакции')
        elif hints[a] and not role:
            role.append('; '.join(sorted(hints[a])))
        src = k.get('source', 'transaction')
        if src != 'transaction':
            role.append('загружен из Address Lookup Table')
        role_s = ' | '.join(role) or 'прочий аккаунт (пул/служебный)'
        data['accounts'].append((i, a, flags, role_s))
        p(f"  [{i:2}] {a:<44} {flags:<16} {role_s}")

    # --- инструкции ---
    inner = {g['index']: g['instructions'] for g in meta.get('innerInstructions') or []}
    n_inner = sum(len(v) for v in inner.values())
    p(f"\nInstructions: {len(msg['instructions'])} верхнего уровня + {n_inner} inner")
    cu_limit = cu_price = None

    def show(ix, label, indent):
        nonlocal cu_limit, cu_price
        t, details, accts = ix_view(ix)
        if t == 'SetComputeUnitLimit':
            cu_limit = details['units']
        if t == 'SetComputeUnitPrice':
            cu_price = details['microLamports']
        d = ', '.join(f'{k}={v}' for k, v in details.items())
        data['ix'].append((label.split(' ')[0], 'inner' in label, ix.get('stackHeight'), pname(ix['programId']), t, d,
                           ', '.join(f'{k}={short(v)}' for k, v in accts)))
        p(f"{indent}{label} {pname(ix['programId'])}: {t}{'  (' + d + ')' if d else ''}")
        if accts:
            p(f"{indent}    accounts: " + ', '.join(f'{k}={short(v)}' for k, v in accts))

    for i, ix in enumerate(msg['instructions']):
        show(ix, f'#{i + 1}', '  ')
        for j, sub in enumerate(inner.get(i, [])):
            show(sub, f'#{i + 1}.{j + 1} [inner, глубина {sub.get("stackHeight", "?")}]', '      ')

    # --- программы ---
    p('\nPrograms:')
    for pid in sorted(invoked, key=pname):
        p(f'  {pid:<44} {pname(pid)}')
    data['programs'] = [(pid, pname(pid)) for pid in sorted(invoked, key=pname)]
    data['n_top'], data['n_inner'] = len(msg['instructions']), n_inner

    # --- комиссия ---
    fee = meta['fee']
    nsig = len(tx['signatures'])
    data.update(fee=fee, nsig=nsig, prio=None, cu_price=cu_price, cu_limit=cu_limit, cu=meta.get('computeUnitsConsumed'))
    base = 5000 * nsig
    p(f"\nFee: {fee} lamports = {sol(fee)} SOL")
    p(f"  базовая: {nsig} подпись(и) × 5000 = {base} lamports")
    if cu_price:
        limit = cu_limit if cu_limit is not None else None
        if limit is not None:
            prio = math.ceil(cu_price * limit / 1_000_000)
            data['prio'] = prio
            data['fee_ok'] = base + prio == fee
            p(f"  priority fee: {cu_price} µlamports/CU × лимит {limit} CU = {prio} lamports"
              f"  (проверка: база + priority = {base + prio} {'== fee ✓' if base + prio == fee else '!= fee — не сходится'})")
        else:
            p(f"  priority fee: цена {cu_price} µlamports/CU, лимит не задан явно (берётся по умолчанию) — сумму не считаю")
    else:
        p('  priority fee: нет (SetComputeUnitPrice отсутствует)')
    if cu_limit is not None:
        p(f"  лимит CU (SetComputeUnitLimit): {cu_limit}")
    p(f"Compute units consumed: {meta.get('computeUnitsConsumed')}")

    # --- балансы ---
    p('\nИзменения SOL (lamports, только изменившиеся):')
    for i, (a, pre, post) in enumerate(zip(addrs, meta['preBalances'], meta['postBalances'])):
        if pre != post:
            data['sol'].append((i, a, pre, post))
            p(f"  [{i:2}] {a:<44} {pre:>16,} → {post:>16,}  ({post - pre:+,} = {'+' if post > pre else ''}{sol(post - pre)} SOL)".replace(',', ' '))
    pre_t = {b['accountIndex']: b for b in meta.get('preTokenBalances') or []}
    post_t = {b['accountIndex']: b for b in meta.get('postTokenBalances') or []}
    rows = []
    for i in sorted(set(pre_t) | set(post_t)):
        b0, b1 = pre_t.get(i), post_t.get(i)
        ref = b1 or b0
        dec = ref['uiTokenAmount']['decimals']
        a0 = int(b0['uiTokenAmount']['amount']) if b0 else 0
        a1 = int(b1['uiTokenAmount']['amount']) if b1 else 0
        f = lambda x: f'{x / 10 ** dec:,.{dec}f}'.rstrip('0').rstrip('.').replace(',', ' ')
        data['tokens'].append((i, addrs[i], mint_label(ref['mint'], network), ref.get('owner', '?'),
                               f(a0) if b0 else '—', f(a1) if b1 else '— (закрыт)', ('+' if a1 >= a0 else '-') + f(abs(a1 - a0))))
        rows.append(f"  [{i:2}] {short(addrs[i])} {mint_label(ref['mint'], network):<22} владелец {short(ref.get('owner', '?'))}: "
                    f"{f(a0) if b0 else '—'} → {f(a1) if b1 else '— (закрыт)'}  ({'+' if a1 >= a0 else '-'}{f(abs(a1 - a0))})")
    p('\nИзменения токенов (pre → post):')
    out.extend(rows or ['  нет токен-балансов'])
    data['mint_fees'] = {mint_label(m, network): bps for m, bps in MINT_FEES.items()}
    for m, bps in MINT_FEES.items():
        p(f"  ! у mint {mint_label(m, network)} есть Token-2022 transfer fee {bps / 100:g}% "
          f"(текущая конфигурация mint) — получатель получает сумму transferChecked минус комиссию")

    logs = meta.get('logMessages') or []
    p(f"\nЛоги: {len(logs)} строк")
    return '\n'.join(out)


def main():
    args = sys.argv[1:]
    if not args:
        sys.exit(__doc__)
    if args[0] == '--file':
        resp = json.load(open(args[1]))
        network = args[2] if len(args) > 2 else 'mainnet'
    else:
        network = args[1] if len(args) > 1 else 'mainnet'
        resp = get_tx(network, args[0])
    if not resp.get('result'):
        sys.exit(f'Транзакция не найдена: {resp}')
    print(analyze(resp, network))


if __name__ == '__main__':
    main()
