"""Минимальный JSON-RPC клиент Solana с повторами при rate limit (только чтение)."""
import json, time, urllib.request, urllib.error

RPC = {
    'mainnet': 'https://api.mainnet-beta.solana.com',
    'devnet': 'https://api.devnet.solana.com',
}


def call(network, method, params, tries=8):
    body = json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': method, 'params': params}).encode()
    delay = 2
    for i in range(tries):
        req = urllib.request.Request(RPC.get(network, network), body, {'Content-Type': 'application/json'})
        try:
            with urllib.request.urlopen(req, timeout=30) as r:
                resp = json.load(r)
            if 'error' in resp and resp['error'].get('code') in (429, -32429):
                raise urllib.error.HTTPError(None, 429, 'rate limit', None, None)
            return resp
        except (urllib.error.HTTPError, urllib.error.URLError, TimeoutError) as e:
            if i == tries - 1:
                raise
            time.sleep(delay)
            delay = min(delay * 2, 30)


def get_tx(network, sig):
    return call(network, 'getTransaction', [sig, {
        'encoding': 'jsonParsed', 'maxSupportedTransactionVersion': 0, 'commitment': 'confirmed'}])
