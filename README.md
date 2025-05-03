# QuantEdge Engine ![WIP](https://img.shields.io/badge/status-WIP-yellow)

🦀 **QuantEdge Engine** is a high-performance and extensible strategy execution engine for quantitative cryptocurrency trading, written in Rust. It powers the backend of the [QuantEdge Platform](https://github.com/8lovelife/quantedge), supporting backtesting, simulation, and portfolio-level analysis.

---

## ✨ Features

- **Modular Strategy Framework**  
  Supports flexible, reusable strategies with configurable parameters. Designed for seamless application across backtesting, paper trading, and live environments.

- **Robust Backtesting Engine**  
  Simulates realistic trading conditions with support for slippage, commission, capital allocation, stop loss, and take profit.

- **Pluggable Indicator System**  
  Includes essential indicators like Moving Average, Bollinger Bands, and Z-Score, with extensibility for custom indicators.

- **Performance Analytics Layer**  
  Evaluates strategy performance with metrics such as Sharpe Ratio, Maximum Drawdown, Win Rate, and Profit Factor.

- **Embedded DuckDB Storage**  
  Stores strategy configurations and backtest results locally using SQL-compatible, high-performance embedded storage.

- **Unified Exchange Abstraction (WIP)**  
  Built for multi-exchange compatibility through a decoupled adapter interface supporting both centralized and decentralized platforms.

## 🚀 Getting Started

### 1. Clone the Repository

```bash
git clone https://github.com/8lovelife/quantedge_x.git
cd quantedge_x
```

### 2. Init the database
```bash
cargo run --bin setup_db
```
### 2. Run the engine as backend

```bash
cargo run --bin quantedge_x
```

---


## 🛣 Roadmap

| Feature                          | Status        |
|----------------------------------|---------------|
| Backtesting Engine               | ✅ Completed   |
| Multi-Asset Support              | ✅ Completed   |
| REST API Server Interface        | ✅ Completed    |
| Exchange Adapter Layer           | 🔄 In Progress |
| Paper/Live Trading Integration   | 🔄 In Progress    |
| Strategy Optimization            | 🔄 In Progress    |


---

## 🔗 Related Projects

- [QuantEdge UI](https://github.com/8lovelife/quantedge) – Frontend interface for building and analyzing strategies.

## 📜 License

MIT License © 2025 [8lovelife]

> QuantEdge Engine is developed for high-performance quantitative research and strategy simulation in cryptocurrency markets.  

