# Kociema
Herbert Kociemba的[The Two-Phase-Algorithm](https://kociemba.org/twophase.htm)的Rust实现（https://kociemba.org/cube.htm），平均小于19步解决一个3x3的魔方。

### 基本设计
1. 在Kewb(https://github.com/luckasRanarison/kewb)的基础上，移植Python版本的官方TwophaseSolver实现(https://github.com/hkociemba/RubiksCube-TwophaseSolver)。
2. 使用lazy_static把相关数据表(SOLVERTABLES)初始化为全局静态变量，以供多线程使用。
3. 多线程（参考Python版本的实现）。
4. 支持超时机制，并且始终有结果返回（即使方案长度大于期望）。
5. 一个简单的命令行工具kociemba-cli.
```
PS C:\Projects\kociemba>kociemba-cli.exe solve --facelet "RLLBUFUUUBDURRBBUBRLRRFDFDDLLLUDFLRRDDFRLFDBUBFFLBBDUF" -m 20 -p
⠇ Solving
Solution:  R D2 B2 R2 L2 B' U F' D2 R B2 R2 F2 B2 R2 D2 B
Move count: 17
Solve time: 3.163ms
Total time: 3.0156508s
```
```
PS C:\Projects\kociemba>kociemba-cli.exe
crate for solving the 3x3 Rubik's cube with Kociemba's two phase algorithm

Usage: kociemba.exe [COMMAND]

Commands:
  solve     solves the cube using two-phase algorithm
  scramble  generates scramble
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
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
5. A simple command-line tool, kociemba, see above.

### References
* Herbert Kociemba的[The Two-Phase-Algorithm](https://kociemba.org/twophase.htm)
* [Kewb](https://github.com/luckasRanarison/kewb)
* [RubiksCube-TwophaseSolver](https://github.com/hkociemba/RubiksCube-TwophaseSolver)
