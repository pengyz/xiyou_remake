# 分发打包规则（Windows）

- 日期：2026-10-18（用户需求：分发给朋友试玩，压缩包即可）
- 实现：`tools/package_win.py`（本规则的可执行真相源；改规则先改脚本再同步本文）

## 包形态

```
xiyou-win-<git短sha>.zip
└── xiyou-win/
    ├── xiyou.exe          # shell/win 二进制（cargo build --release）
    ├── assets/raw/**      # 游戏资源（extract.py 产物）
    ├── assets/audio/**    # BGM OGG
    └── README.txt         # 运行/操作/存档/参数说明（脚本生成）
```

## 规则要点

1. **exe 与 assets/ 同级**——游戏运行时默认取 cwd 相对 `assets/raw`、`assets/audio`
   （main.rs 参数区），存档默认 `exe目录\save`（决策：decision_save-path-shell-injection）
   ⇒ 双击即跑、零配置、存档随目录可拷走续档。
2. **分发红线（2026-10-18 用户指正）**：`*.class`（a.class/CMidlet.class）与
   `*.MF`（JAR 清单）**绝不入包**——原版字节码=代码面、运行时零读取；assets/raw
   内保留仅供 reference/oracle 工具链。打包步骤黑名单剔除 + 把关自检（包内出现
   即打包失败）。
2. **不捆绑字体**——中文字体走 `%WINDIR%\Fonts` 系统字体（font_inject 发现）；
   winmm/winit/softbuffer 均系统或静态链接，无红依赖。
3. **资产源自原版 JAR**——README 内含"仅供私人分享试玩"声明；分发物不含
   original/ 归档与仓库其余部分（代码以二进制形态）。
4. **版本标识** = 打包时 git 短 sha（可追溯到提交）；`--skip-build` 复打包既有产物。

## 用法

```bash
python tools/package_win.py              # 构建+打包 → dist/xiyou-win-<sha>.zip
python tools/package_win.py --skip-build # 只打包（复用 target/release 产物）
```

## 验收（发给朋友前）

1. 解压 zip 到新目录，双击 xiyou.exe 能进主菜单；
2. 进游戏走两步、听 BGM 正常；
3. 保存→退出→重开，读档列表可见（save\ 出现在 exe 旁）。
