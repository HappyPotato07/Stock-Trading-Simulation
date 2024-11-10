Stock trading is a domain where real-time systems (RTS) play a critical role in processing high-frequency transactions with minimal delay. Trading decisions must be executed promptly to capitalize on market opportunities or minimize potential losses. In this context, the Real-Time Stock Trading System Simulation, built using the Rust programming language, explores how high-performance and concurrency features can meet the demands of real-time stock market operations.

The primary goal of this project is to simulate a real-time trading environment that captures the complexities of efficient resource utilization, predictable system performance, and reliable trade execution. The system incorporates subsystems for Trade Processing and Stock Market Processing, which collaborate to manage trading algorithms, execute orders, and process market data. It also integrates RabbitMQ for asynchronous messaging to ensure seamless communication between traders and brokers.

Key Questions:
I. How can Rust's asynchronous programming and concurrency features improve the trading system's ability to handle high-frequency trades efficiently and reliably?
II. How do Rust's memory safety and ownership principles ensure system reliability, data integrity, and robustness in managing concurrent trading operations?


# Design Flow
![image](https://github.com/user-attachments/assets/f0e6df4e-b155-49a7-9a08-390b6b7e1bd9)
