curl -X POST http://localhost:3001/api/backtest/run \
-H "Content-Type: application/json" \
-d '{
"params": {
"smaFast": 10,
"smaSlow": 50,
"riskLevel": "medium",
"stopLoss": 100,
"takeProfit": 200,
"useTrailingStop": true,
"trailingStopDistance": 50
},
"timeframe": "1h",
"strategyId": 1
}'


curl -X GET 'http://localhost:3001/api/backtest/history?strategyId=123'

curl -X GET 'http://localhost:3001/api/backtest?version=45&strategyId=1'

curl -X GET 'http://localhost:3001/api/algorithms'

curl -X GET 'http://localhost:3001/api/strategies/details?strategyId=9'

websocat "ws://localhost:3001/ws?exchange=QuantEdge&symbol=BTC/USDT&interval_ms=300"
