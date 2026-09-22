# 旧版 MkDocs 网站归档

此目录保存迁移前的 MkDocs 网站实现，供回滚和对照使用。仓库根目录的 Markdown、Excel
等仍是资料源，旧站构建脚本会从根目录读取这些内容，不在此目录复制一份。

Cloudflare Pages 可将此目录设为 Root directory，构建命令为 `bash build-site.sh`，
输出目录为 `site`；这样 `functions/` 下的旧站 API 代理也会继续生效。

构建入口：

```bash
bash build-site.sh
```

Windows 本地预览：

```powershell
.\build-site.ps1 -Serve
```

输出目录为 `site/`。该归档不再作为生产站点部署入口；新站位于仓库根目录的 `website/`。
