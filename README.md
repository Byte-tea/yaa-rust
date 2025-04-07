<h1 align="center">
  <a href="https://github.com/Byte-tea/yaa/">
    <img src="assets/yaa.svg" width="150" height="150" alt="banner" /><br>
  </a>
</h1>

<p align="center">中文 | <a href="README.en.md">English</a></p>

## yaa

`yaa` 的意思是 Yet Another Agent。是一个能分析理解自然语言指示，自动创建、规划、执行、验证任务的智能体。

## 快速上手

### 部署

拉取到本地：

```bash
# 拉取项目仓库到本地
git clone https://github.com/Byte-tea/yaa.git
# 进入项目目录
cd yaa
```

配置：

```bash
# 创建客户端相关配置文件，传入时会覆盖默认配置
cp configs/Config.example.json Config.json
# 创建服务端相关配置文件，传入时会覆盖默认配置
cp configs/ServerConfig.example.json ServerConfig.json
```

### 使用

以服务模式启动：

```bash
cargo run --release -- --serve --port 12345
```

以命令行交互模式启动：

```bash
cargo run --release
> 请用 Python 写一个冒泡排序算法。
* 好的……
```

## 项目实现

详见 [yaa 文档](docs/README.md)。
