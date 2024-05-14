# Kociema
Herbert Kociemba的The Two-Phase-Algorithm的Rust实现（https://kociemba.org/cube.htm，https://kociemba.org/twophase.htm）。

## 基本设计
1. 在Kewb(https://github.com/luckasRanarison/kewb)的基础上，移植Python版本的官方实现(https://github.com/hkociemba/RubiksCube-TwophaseSolver)。
2. 使用lazy static把相关数据表(SOLVERTABLES)初始化为全局静态变量，以供多线程使用。
3. 多线程（参考Python版本的实现）。
4. 支持超时机制（有Bug，包括了数据表加载时间）。

## TODO
1. 超时机制不是真正的算法执行时间，包括了1.2x秒的数据表加载时间(目前的开发环境)。
~~2. 多次加载数据表问题。~~
3. solutions第一个空元素问题。
4. lazy static是否可优化。
5. 命令行程序移植。
6. 代码清理，完善注释文档。
7. 发布到crates.io。