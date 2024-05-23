# Kociema
Herbert Kociemba的[The Two-Phase-Algorithm](https://kociemba.org/twophase.htm)的Rust实现（https://kociemba.org/cube.htm），平均小于19步解决一个3x3的魔方。

### 基本设计
1. 在Kewb(https://github.com/luckasRanarison/kewb)的基础上，移植Python版本的官方TwophaseSolver实现(https://github.com/hkociemba/RubiksCube-TwophaseSolver)。
2. 使用lazy_static把相关数据表(SOLVERTABLES)初始化为全局静态变量，以供多线程使用。
3. 多线程（参考Python版本的实现）。
4. 支持超时机制，并且始终有结果返回（即使方案长度大于期望）。
5. 一个简单的命令行工具kociemba-cli.

```
PS C:\Projects\kociemba>cargo run -p kociemba-cli solve --facelet "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF" -m 20 -p
⠇ Solving
Solution:  R D2 B2 R2 L2 B' U F' D2 R B2 R2 F2 B2 R2 D2 B
Move count: 17
Solve time: 3.163ms
Total time: 3.0156508s
```
```
PS C:\Projects\kociemba>cargo run -p kociemba-cli
solving the 3x3 Rubik's cube with Kociemba's two phase algorithm

Usage: kociemba-cli.exe [COMMAND]

Commands:
  solve     solves the cube using two-phase algorithm
  scramble  generates scramble
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
6. Web GUI(基于yew.rs)和http server.
```
PS C:\Projects\kociemba> cargo run -p kociemba-server
   Compiling kociemba v0.5.2 (C:\Projects\kociemba)
   Compiling kociemba-server v0.5.2 (C:\Projects\kociemba\server)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.13s
     Running `target\debug\kociemba-server.exe`
listening on http://127.0.0.1:32125
```
```
Solve a cube: http://localhost:32125/solve/

Example: http://localhost:32125/solve/DUUBULDBFRBFRRULLLBRDFFFBLURDBFDFDRFRULBLUFDURRBLBDUDL
Get a scramble: http://localhost:32125/scramble
```
```
PS C:\Projects\kociemba\web> trunk serve --open -A ipv4
2024-05-23T13:10:30.611015Z  INFO 🚀 Starting trunk 0.20.1
2024-05-23T13:10:30.615589Z  INFO 📦 starting build
... ... ...
2024-05-23T13:10:31.230370Z  INFO applying new distribution
2024-05-23T13:10:31.234240Z  INFO ✅ success
2024-05-23T13:10:31.234359Z  INFO 📡 serving static assets at -> /
2024-05-23T13:10:31.235899Z  INFO 📡 server listening at:
2024-05-23T13:10:31.235972Z  INFO     🏠 http://127.0.0.1:8080/
```

### Crates.io
* https://crates.io/crates/kociemba
### github
* https://github.com/adungaos/kociema
### TODO
~~1. 超时机制不是真正的算法执行时间，包括了1.2x秒的数据表加载时间(目前的开发环境)。~~
  * solers.rs中solver两次循环，第一次加载数据表并完成一个预置魔方的解答，第二次才真正解决用户的输入。

~~2. 多次加载数据表问题。~~

3. solutions第一个空元素问题。

4. lazy static是否可优化。

~~5. 命令行程序移植。~~

6. 代码清理，完善注释文档。

~~7. 发布到crates.io。~~

8. 错误处理。


### 参考资料
* Herbert Kociemba的[The Two-Phase-Algorithm](https://kociemba.org/twophase.htm)
* [Kewb](https://github.com/luckasRanarison/kewb)
* [RubiksCube-TwophaseSolver](https://github.com/hkociemba/RubiksCube-TwophaseSolver)

------

# English

## Kociema
The Rust implementation of Herbert Kociemba's [Two-Phase-Algorithm](https://kociemba.org/twophase.htm),(https://kociemba.org/cube.htm), to solving a 3x3 Rubik's cube less than 19 moves on average.

### Brief
1. Based on [Kewb](https://github.com/luckasRanarison/kewb) and the official TwophaseSolver implementation of Python version (https://github.com/hkociemba/RubiksCube-TwophaseSolver).
2. Use lazy_static to initialize the relevant data tables (SOLVERTABLES) as a global static variable for multithreading.
3. Multithreadings (reference to implementation of Python version).
4. Support the timeout mechanism and always return results (even if the move length is longer than expected).
5. A simple command-line tool, kociemba-cli, see above.
6. A web GUI(powered by yew.rs) and a http server, see above.

### References
* Herbert Kociemba的[The Two-Phase-Algorithm](https://kociemba.org/twophase.htm)
* [Kewb](https://github.com/luckasRanarison/kewb)
* [RubiksCube-TwophaseSolver](https://github.com/hkociemba/RubiksCube-TwophaseSolver)
