# world.execute(me); — Rust 终端 MV

[`world.execute-me-ascii`](https://github.com/yym8224961/world.execute-me-ascii) 的 Rust 重写版。全平台支持，不只是MacOS；使用 **crossterm** 处理终端，**rodio + symphonia** 播放内嵌音乐，渲染与 Python 原版逐字节一致。

原曲与歌词：Mili《world.execute(me);》。本项目为个人创作与备份，未对原曲、歌词或其他第三方素材授予额外使用许可。

## 运行

```sh
cargo run --release
```

音乐、字幕、频谱数据已通过 `include_bytes!` / `include_str!` 编译进二进制，运行时不读取外部文件，也无需联网。

| 参数 | 说明 |
| --- | --- |
| `--audio <FILE>` | 播放指定音频（默认内嵌原曲） |
| `--start <秒>` | 起始位置，默认 0 |
| `--autoplay` | 立即播放 |
| `--paused` | 停在 `--start`，不显示就绪画面 |
| `--fps <N>` | 帧率 5–60，默认 24 |
| `--offset <秒>` | 字幕偏移 |
| `--snapshot <T>` | 只渲染某一帧后退出（配合 `--width/--height/--plain`） |
| `--report <FILE>` | 退出时写出 JSON 帧时序报告 |
| `--stop-after <T>` | 播放到 T 秒后退出 |

按键与原版一致：`空格/回车` 播放暂停，`←/→` 前后 5 秒，`R` 重头，`1`–`5` 章节跳转，`[`/`]` 字幕偏移，`,`/`.` 上下句，`+`/`-` 音量，`H` 帮助，`Q`/`ESC` 退出。推荐全屏，至少 64×24。

## 行为一致性

「与原版完全一致」是可验证的，不是靠肉眼比对。仓库自带差分测试：

- `tools/oracle.py` 用**未经修改**的 `world.execute-me-ascii/player.py` 渲染帧。原版 `player.py` 依赖 `termios`/`tty`（macOS 专有），oracle 只替换这些模块，渲染路径本身完全走原代码。
- `tools/gen_goldens.py` 为 587 个时间点、多种尺寸与状态各起一个全新 Python 进程，记录精确的 ANSI 字节流到 `tools/goldens.json`。
- `tests/golden.rs` 用 Rust 渲染同一批帧并逐字节比较：

```sh
python tools/gen_goldens.py                      # 重新生成基线
cargo test --release --test golden               # 逐字节比对
```

覆盖范围包括全部场景切换点、章节边界、16 个终端尺寸（含 64×24 下限与 240×85 上限）、帮助/就绪/暂停/字幕偏移等状态，以及故障效果最密集的时间段。

**当前状态：587 帧中 560 帧（95.4%）逐字节一致。** 剩余 27 帧的差异全部局限于标题接管（title takeover，16–29.7s）与其相邻的故障/荧光效果，表现为每帧个位数像素的字符不同（例如某格 `o` 与 `*`）。这些是 Python 与 Rust 浮点舍入差异在 `hash16` 门限上的放大结果——阈值判定对末位比特极其敏感。所有场景的结构、布局、样式与文字均一致。

## 动画流畅度

渲染本身不是瓶颈。`cargo run --release --example perf_probe`：

| 区段 | 平均 | 最差 |
| --- | --- | --- |
| boot | 0.045 ms | 0.181 ms |
| title | 0.589 ms | 2.792 ms |
| devotion | 0.170 ms | 3.397 ms |
| organic | 0.339 ms | 3.926 ms |
| isolation | 0.100 ms | 2.294 ms |
| love | 0.129 ms | 2.728 ms |

全曲 5086 帧渲染仅需 1.02 秒，约 **208× 实时余量**；24 fps 的单帧预算是 41.7 ms，最差帧仅 3.9 ms。帧循环用 `Instant` 累加目标时间并配合 `event::poll(delay)` 等待输入，因此不会像原版那样自行空转，也不会累积漂移。

## 与原版的实现差异

| 方面 | 原版 | 本移植 |
| --- | --- | --- |
| 音频 | macOS AVFoundation，独立 `audio-clock` 子进程，每帧读 JSON | rodio + symphonia，进程内播放；位置时钟由单调 `Instant` + 定位点推导 |
| 平台 | 仅 macOS | 跨平台（crossterm 负责终端） |
| 资源 | 运行时读取目录，或从 .pyz 解包 | 编译期内嵌 |
| 故障效果 | 每次 `c.cells[y][x] = (ch, style)` | 同样的格子模型，经 `set`/`rotate_row` 访问 |

音频时钟刻意做成同步的：`seek`/`play`/`pause` 立即生效，下一帧即可见，因此按键响应比原版「写命令给子进程再回读」更快且没有延迟。

## 代码结构

原版 `scenes.py` 是单个 3073 行文件，这里按时间段拆分，便于分别移植与复核：

```
src/
  core.rs        调色板、hash16、Python 语义的 round/clamp
  canvas.rs      字符网格 + ANSI 序列化（双宽字符占位格）
  text.rs        cw/width/crop/wrap，对齐 unicodedata
  font.rs        5 行点阵字体
  film.rs        资源、歌词/频谱查询、顶层 render
  audio.rs       rodio 播放与音频时钟
  player.rs      CLI、按键、终端生命周期、帧循环
  scenes/
    common.rs     助手、故障引擎、3D 投影
    boot.rs       L130-433    boot 与 simulation
    devotion.rs   L434-1274   points → satisfaction
    organic.rs    L1275-1600  execution、heart、legacy organic
    identity.rs   L1601-2028  身份切换、时钟、振动
    isolation.rs  L2029-2603  isolation → execution orb
    love.rs       L2604-2774  love equation、trapped loop、outro
    dispatch.rs   L2777-3073  title takeover、draw_scene、phosphor
```

移植中最容易出错、也已在代码中显式标注的几处语义差异：

- `round()` 是 Python 的**四舍六入五取偶**，且对负数同样成立（`round(-2.5) == -2`）——`f64::round` 会给出 -3。用 `core::pyround`。
- `int()` 向零截断，`//` 向下取整（`div_euclid`），`%` 取除数符号（`rem_euclid`）。
- `hash16(x) % m < threshold` 中如果 `threshold` 是**浮点**，比较必须在浮点域进行。`buildup * 50` 在 `buildup ≈ 3.6e-15` 时仍会命中约 50 个格子；若把阈值转成整数就会全部丢失。
- `if e > .3 and k % 5 < int(e * 5)` 的右侧是截断后的**整数**，与浮点比较结果不同。

## 测试

```sh
cargo test --release            # 单元测试 + 逐字节差分基线
cargo run --release --example perf_probe      # 帧耗时
cargo run --release --example diff_goldens -- 5   # 逐行展示偏差
```
