#!/usr/bin/env python3
"""Собирает report_week2.docx. Таблицы и цифры берутся из tx*.json через analyze.py."""
import json
from docx import Document
from docx.shared import Pt, RGBColor
from docx.oxml.ns import qn
from docx.oxml import OxmlElement
from docx.opc.constants import RELATIONSHIP_TYPE as RT

from analyze import analyze, sol

SIGS = {n: (net, s) for n, net, s in (l.split() for l in open('signatures.txt'))}


def load(name):
    net = SIGS[name][0]
    d = {}
    analyze(json.load(open(f'{name}.json')), net, d)
    d['net'] = net
    return d


T = {n: load(n) for n in ('tx0', 'tx1', 'tx2', 'tx3')}
sp = lambda n: f'{n:,}'.replace(',', ' ')

doc = Document()
st = doc.styles['Normal']
st.font.name = 'Calibri'
st.font.size = Pt(12)
st.element.rPr.rFonts.set(qn('w:eastAsia'), 'Calibri')
for s, size in (('Title', 16), ('Heading 1', 14), ('Heading 2', 13)):
    h = doc.styles[s]
    h.font.color.rgb = RGBColor(0, 0, 0)
    h.font.size = Pt(size)
    rf = h.element.rPr.rFonts
    for a in ('w:asciiTheme', 'w:hAnsiTheme', 'w:eastAsiaTheme', 'w:cstheme'):
        rf.attrib.pop(qn(a), None)
    rf.set(qn('w:ascii'), 'Calibri')
    rf.set(qn('w:hAnsi'), 'Calibri')
pPr = doc.styles['Title'].element.find(qn('w:pPr'))
if pPr is not None and pPr.find(qn('w:pBdr')) is not None:
    pPr.remove(pPr.find(qn('w:pBdr')))


def para(text, style=None):
    return doc.add_paragraph(text, style=style)


def bullet(text):
    return doc.add_paragraph(text, style='List Bullet')


def link(label, url):
    p = doc.add_paragraph()
    p.add_run(label)
    rid = p.part.relate_to(url, RT.HYPERLINK, is_external=True)
    h = OxmlElement('w:hyperlink')
    h.set(qn('r:id'), rid)
    r = OxmlElement('w:r')
    rpr = OxmlElement('w:rPr')
    u = OxmlElement('w:u')
    u.set(qn('w:val'), 'single')
    rpr.append(u)
    r.append(rpr)
    t = OxmlElement('w:t')
    t.text = url
    r.append(t)
    h.append(r)
    p._p.append(h)


def table(header, rows, size=9):
    t = doc.add_table(rows=1, cols=len(header))
    t.style = 'Table Grid'
    for c, txt in zip(t.rows[0].cells, header):
        c.text = ''
        c.paragraphs[0].add_run(txt).bold = True
    for row in rows:
        cells = t.add_row().cells
        for c, txt in zip(cells, row):
            c.text = str(txt)
    for row in t.rows:
        for c in row.cells:
            for p in c.paragraphs:
                for r in p.runs:
                    r.font.size = Pt(size)
    doc.add_paragraph()


def explorer(name):
    net, sig = SIGS[name]
    return f'https://explorer.solana.com/tx/{sig}' + ('?cluster=devnet' if net == 'devnet' else '')


def tx_section(name, title, what):
    d = T[name]
    doc.add_heading(title, level=1)
    para(f"Signature: {d['signature']}")
    link('Explorer: ', explorer(name))
    para(f"Slot {d['slot']}, {d['time']}, версия транзакции: {d['version']}. "
         f"Fee payer: {d['fee_payer']}. Signers: {len(d['signers'])}.")

    doc.add_heading(f"Accounts ({len(d['accounts'])})", level=2)
    table(['#', 'Адрес', 'Права', 'Роль'], [(i, a, f.replace(',', ', '), r) for i, a, f, r in d['accounts']], size=8)

    doc.add_heading(f"Instructions ({d['n_top']} верхнего уровня + {d['n_inner']} inner)", level=2)
    rows = []
    for label, inner, depth, prog, typ, det, accts in d['ix']:
        n_acc = accts.count('=')
        acc = accts if n_acc <= 6 else f'{n_acc} аккаунтов (см. analysis.txt)'
        rows.append((label + (f' (inner, ур. {depth})' if inner else ''), prog, typ, det, acc))
    table(['№', 'Программа', 'Тип', 'Параметры', 'Accounts'], rows, size=8)

    doc.add_heading('Programs', level=2)
    for pid, pn in d['programs']:
        bullet(f'{pn} — {pid}')

    doc.add_heading('Комиссия и compute units', level=2)
    fee_line = f"Fee: {sp(d['fee'])} lamports = {sol(d['fee'])} SOL (база {d['nsig']} × 5 000"
    if d['prio'] is not None:
        fee_line += (f" + priority {sp(d['prio'])}: {sp(d['cu_price'])} µlamports/CU × лимит {sp(d['cu_limit'])} CU; "
                     f"сумма сходится с полем fee)")
    else:
        fee_line += ', priority fee нет)'
    bullet(fee_line)
    lim = f" из лимита {sp(d['cu_limit'])}" if d['cu_limit'] else ' (лимит не задан, по умолчанию 200 000 на инструкцию)'
    bullet(f"Compute units consumed: {sp(d['cu'])}{lim}.")

    doc.add_heading('Изменения балансов', level=2)
    table(['#', 'Адрес', 'SOL до, lamports', 'SOL после, lamports', 'Изменение'],
          [(i, a, sp(pre), sp(post), f'{post - pre:+,}'.replace(',', ' ')) for i, a, pre, post in d['sol']], size=8)
    if d['tokens']:
        table(['#', 'Токен-аккаунт', 'Токен', 'Владелец', 'До', 'После', 'Изменение'],
              [(i, a[:4] + '…' + a[-4:], m, o[:4] + '…' + o[-4:], b0, b1, ch) for i, a, m, o, b0, b1, ch in d['tokens']], size=8)

    doc.add_heading('Что произошло', level=2)
    for s in what:
        para(s)


t1, t2, t3, t0 = T['tx1'], T['tx2'], T['tx3'], T['tx0']
lam1 = 1000  # из tx1.json: transfer lamports
assert t1['sol'][1][3] - t1['sol'][1][2] == lam1

doc.add_heading('Задание 2. Как на самом деле работает Solana: анализ транзакций', level=0)
para('Сеть: Solana mainnet-beta (tx1–tx3), devnet (бонус). Данные получены через JSON-RPC getTransaction '
     '(encoding jsonParsed, maxSupportedTransactionVersion 0) и разобраны скриптом analyze.py. Все цифры ниже взяты из tx*.json.')

doc.add_heading('Краткая теория', level=1)
theory = [
    ('Accounts Model', 'Всё состояние Solana хранится в аккаунтах: у каждого есть адрес, баланс в lamports, данные и программа-владелец. '
     'Менять данные и списывать lamports аккаунта может только его владелец; кошелёк — это аккаунт, которым владеет System Program. '
     'Чтобы аккаунт жил, на нём держат депозит под rent (rent-exempt minimum).'),
    ('Programs', 'Программы — это аккаунты с исполняемым кодом (смарт-контракты), сами они не хранят состояние, а читают и пишут чужие аккаунты. '
     'Есть встроенные (System, Compute Budget) и пользовательские программы (SPL Token, Jupiter, Raydium).'),
    ('Instructions', 'Инструкция — вызов одной программы: program id, список аккаунтов с флагами signer/writable и байты данных. '
     'Программа может вызвать другую программу (CPI) — такие вызовы видны в мета-данных как inner instructions.'),
    ('Transactions', 'Транзакция — набор инструкций, подписей и recent blockhash; выполняется атомарно: либо все инструкции, либо ни одной. '
     'Заранее перечислены все аккаунты, поэтому транзакции с непересекающимися writable-аккаунтами исполняются параллельно. '
     'Версия 0 позволяет подгружать адреса из Address Lookup Tables, чтобы уместить больше аккаунтов.'),
    ('Compute Units и комиссии', 'Каждая инструкция расходует compute units (CU); лимит на транзакцию — 1 400 000 CU, по умолчанию 200 000 на инструкцию. '
     'Базовая комиссия — 5 000 lamports за подпись; priority fee = цена за CU (SetComputeUnitPrice, в micro-lamports) × запрошенный лимит CU (SetComputeUnitLimit). '
     'Priority fee повышает шансы попасть в блок при нагрузке.'),
    ('Proof of History', 'PoH — последовательная цепочка SHA-256 хэшей, которая служит криптографическими «часами»: по ней можно доказать порядок и интервал событий. '
     'Это позволяет лидеру упорядочивать транзакции без предварительного согласования времени с другими узлами.'),
    ('Валидаторы и консенсус', 'Валидаторы по расписанию становятся лидерами (слот ≈ 400 мс), собирают блоки, остальные их проверяют и голосуют весом застейканного SOL. '
     'Исторически консенсус — Tower BFT поверх PoH; в 2025 году сообщество приняло протокол Alpenglow (Votor/Rotor), который должен заменить Tower BFT. '
     '[Не проверял: активирован ли Alpenglow на mainnet на дату отчёта.]'),
]
for head, text in theory:
    p = doc.add_paragraph()
    p.add_run(head + '. ').bold = True
    p.add_run(text)

tx_section('tx1', 'tx1 — простой перевод SOL', [
    f"Кошелёк {t1['fee_payer']} одной инструкцией System Program transfer перевёл {lam1} lamports (0.000001 SOL) на {t1['sol'][1][1]}. "
    f"С отправителя списано {sp(-(t1['sol'][0][3] - t1['sol'][0][2]))} lamports: перевод плюс комиссия 5 000.",
    'Priority fee не указана, Compute Budget инструкций нет; потрачено 150 CU — минимальная стоимость перевода. '
    'Цель такого микро-перевода из JSON не видна [не проверял, что это за кошельки].',
])

tx_section('tx2', 'tx2 — перевод SPL-токена (USDC)', [
    f"Владелец {t2['fee_payer'][:4]}…{t2['fee_payer'][-4:]} перевёл 11.765859 USDC со своего токен-аккаунта 8g1g…4hND на токен-аккаунт F3GM…bxQA, "
    'принадлежащий кошельку 51SY…uyZ5 (инструкция transferChecked SPL Token Program, проверяет mint и decimals = 6).',
    'Перед этим Associated Token Account Program вызвал createIdempotent — «создай ATA получателя, если его нет». '
    'Аккаунт уже существовал (он есть в preTokenBalances с нулём, inner instructions нет), поэтому инструкция ничего не создала. '
    'Memo v2 добавил к транзакции текст «1790863505434544605» [похоже на идентификатор вывода с биржи — предположение, не проверено].',
    f"Комиссия {sp(t2['fee'])} lamports = 5 000 база + 294 priority; лимит CU задан ровно под потребление ({sp(t2['cu'])} CU). "
    'SOL меняется только у fee payer — на комиссию; сами токены двигаются между токен-аккаунтами.',
])

tx_section('tx3', 'tx3 — swap через Jupiter Aggregator v6', [
    'Пользователь 4f7v…nRmT обменял 0.013123964 SOL на токен banked по маршруту из двух шагов, который построил Jupiter (инструкция route_v2): '
    'SOL оборачивается в wSOL во временном токен-аккаунте (createIdempotent → transfer → syncNative), затем '
    '1) Raydium CLMM swap_v2: 0.013123964 wSOL → 0.0076 TTWO; 2) Raydium LaunchLab buy_exact_in: 0.0076 TTWO → 309 789.196154 banked. '
    'TTWO — Token-2022 токен, в on-chain метаданных названный «Take-Two Interactive Software - Backpack Securities».',
    'У mint banked включён Token-2022 transfer fee 1% (100 bps): из 309 789.196154 удержано 3 097.891962, '
    'пользователь получил 306 691.304192 — это видно в postTokenBalances. '
    'Из 0.0076 TTWO, оплаченных в пул LaunchLab, 0.000076 (1%) ушло на аккаунт DHEG…bCa2 '
    '[какая именно это комиссия — protocol/platform/creator — не проверял].',
    'Инструкции с типом «emit event» — self-CPI Anchor: программы (Jupiter SwapsEvent, LaunchLab TradeEvent) записывают события в inner instructions, '
    'чтобы индексаторы могли их прочитать. Имена определены по sha256-дискриминатору и совпадают с подписями Explorer.',
    f"SOL пользователя уменьшился на {sp(-(t3['sol'][0][3] - t3['sol'][0][2]))} lamports = 13 123 964 (обмен) + 1 559 560 и 1 574 800 "
    f"(rent двух новых ATA для TTWO и banked) + {sp(t3['fee'])} (комиссия). Rent временного wSOL-аккаунта (1 488 440) вернулся при closeAccount.",
    'Транзакция версии 0: 14 из 36 аккаунтов подгружены из Address Lookup Table. '
    'В SetComputeUnitLimit передан readonly-аккаунт jitodontfront… '
    '[по документации Jito это маркер защиты от фронтраннинга в бандлах — не проверял].',
])

doc.add_heading('Сравнение tx1 (перевод) и tx3 (swap)', level=1)
table(['Параметр', 'tx1 перевод', 'tx3 swap'], [
    ('Accounts', len(t1['accounts']), f"{len(t3['accounts'])} (14 из ALT)"),
    ('Instructions верхнего уровня', t1['n_top'], t3['n_top']),
    ('Inner instructions', t1['n_inner'], t3['n_inner']),
    ('Programs', len(t1['programs']), len(t3['programs'])),
    ('Compute units', sp(t1['cu']), sp(t3['cu'])),
    ('Fee, lamports', sp(t1['fee']), f"{sp(t3['fee'])} (из них priority {sp(t3['prio'])})"),
    ('Версия', t1['version'], t3['version']),
], size=10)
para(f"Swap дороже по вычислениям примерно в {round(t3['cu'] / t1['cu'])} раз: он вызывает 8 программ, двигает токены через два пула "
     f"и создаёт/закрывает аккаунты. Комиссия же выросла лишь в {t3['fee'] / t1['fee']:.1f} раза: базовая часть одинакова (одна подпись), "
     "а priority fee считается от запрошенного лимита CU (1 400 000), а не от фактически израсходованных.")

doc.add_heading('Бонус: моя транзакция из недели 1 (devnet)', level=1)
para(f"Signature: {t0['signature']}")
link('Explorer: ', explorer('tx0'))
para(f"Та же структура, что у tx1: 3 аккаунта, одна инструкция System Program transfer на 100 000 000 lamports (0.1 SOL) "
     f"с моего CLI-кошелька {t0['fee_payer']} на Phantom-адрес; fee {sp(t0['fee'])} lamports, {t0['cu']} CU, priority fee нет. "
     f"Slot {t0['slot']}, {t0['time']}.")

doc.add_heading('Вывод', level=1)
para('Любая транзакция Solana — это список инструкций к программам плюс заранее объявленный список аккаунтов с правами signer/writable; '
     'именно по нему видно, что транзакция может изменить. Простой перевод SOL — одна инструкция System Program и 150 CU, '
     'перевод токена — уже работа с токен-аккаунтами через SPL Token и ATA, а swap — цепочка CPI: Jupiter вызывает пулы Raydium, '
     'те — Token и Token-2022, что видно только в inner instructions. Комиссия складывается из базовых 5 000 lamports за подпись и priority fee '
     '(цена CU × лимит CU), поэтому сложная транзакция стоит дороже прежде всего из-за запрошенного лимита. '
     'Реальное движение средств надёжнее всего проверять по pre/post балансам: в tx3 именно они показали скрытую 1% комиссию Token-2022.')

doc.save('report_week2_full.docx')
print('saved report_week2_full.docx')
