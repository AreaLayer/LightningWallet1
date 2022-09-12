import { NATIVE, BTC_TOKENS } from './COINS'

export default [
  'BTC⚡️ -BTC',

  ...Object.values(BTC_TOKENS).map(token => `{BTC⚡️ }${token}-BTC`),
  // ...Object.values(ETH_TOKENS).map(token => `${token}-USDT`),
]
