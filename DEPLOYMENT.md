# 部署说明

## 新站（主站）

Cloudflare Pages 连接本仓库后，配置：

- Root directory: `website`
- Build command: `bash scripts/build-cloudflare.sh`
- Build output directory: `dist`
- `PUBLIC_URL`: 生产域名带结尾斜杠，例如 `https://example.com/`

每次推送到生产分支后，Cloudflare 会重新构建并发布 `website/dist/`。

新站 `assets/site.json` 中的 `links.legacy_site_url` 默认指向旧站：
`https://iscas-application-guide.pages.dev/`。如果旧站改用自定义域名，只需修改该字段。

## 旧站（并行保留）

旧 MkDocs 实现位于 `legacy/mkdocs-site/`。建议保留一个独立的 Cloudflare Pages 项目，
不要把旧站嵌入新站的 `/old/` 路径。

旧站 Pages 项目使用 `legacy/mkdocs-site` 作为 Root directory：

- Build command: `bash build-site.sh`
- Build output directory: `site`

它仍从仓库根目录读取原始资料，构建命令为：

```bash
bash legacy/mkdocs-site/build-site.sh
```

旧站项目可以继续使用原来的 `iscas-application-guide.pages.dev` 地址；新站项目绑定主域名。
旧站归档配置中的 `extra.new_site_url` 预留了从旧站返回新站的入口，待新站正式域名确定后填写。

## 本地旧站预览

Windows 下可以继续运行仓库根目录的 `build-site.ps1`，它会转调归档目录中的构建脚本；
也可以直接运行 `legacy/mkdocs-site/build-site.ps1`。
