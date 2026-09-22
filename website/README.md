# 新版 ISCAS 指南站

该目录来自 `bigsaltyfishes/ISCAS-Demo` 的 `src` 分支，当前导入提交为
`ad4860cde210b4b7ea1874209aeb3abce4eff27c`。

网站文章位于 `assets/_assets/articles/`，是面向展示层的内容快照；仓库根目录的
Markdown 和数据文件仍是资料维护入口。新增或修订资料后，需要同步更新网站快照。

## Cloudflare Pages

将 Pages 项目的根目录设为 `website`，使用以下配置：

```text
Build command: bash scripts/build-cloudflare.sh
Build output directory: dist
```

构建脚本固定使用 `bigsaltyfishes/trunk` 的 `feat/gzip` 提交
`4758424b9c026b79cfe94fde1ef318df4f9c9216`，并生成 `dist/` 静态产物。

Cloudflare 构建环境必须提供 Node/npm、Rust/cargo、NuShell 和 zstd。若 Pages
构建镜像缺少其中任意工具，构建会明确失败；此时应改用 GitHub Actions 构建后上传
Cloudflare Pages，而不是使用未固定版本的工具继续构建。

新版导航栏中的“旧版站点”地址来自 `assets/site.json` 的
`links.legacy_site_url` 字段，旧站地址变化时只需修改该字段。

`_redirects` 用于支持 `/articles/...` 等前端路由的直接访问和刷新。`functions/`
目录保留旧站的 `/api/pledge` 同源代理，当前新版前端默认使用本地存储状态，后续如
重新启用服务端计数可直接复用该 Function。
