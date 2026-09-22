# 旧版 ISCAS MkDocs 网站的 Windows 本地构建脚本。

param(
    [switch]$Serve
)

$ErrorActionPreference = 'Stop'
$legacyRoot = $PSScriptRoot
$repoRoot = (Resolve-Path (Join-Path $legacyRoot '..\..')).Path
$docs = Join-Path $legacyRoot 'docs_src'
$site = Join-Path $legacyRoot 'site'
$venvPy = Join-Path $repoRoot '.venv\Scripts\python.exe'

if (-not (Test-Path $venvPy)) {
    $venvPy = Join-Path $legacyRoot '.venv\Scripts\python.exe'
}

if (-not (Test-Path $venvPy)) {
    Write-Host '首次运行：创建虚拟环境并安装 mkdocs-material ...'
    $venvDir = Join-Path $repoRoot '.venv'
    python -m venv $venvDir
    $venvPy = Join-Path $venvDir 'Scripts\python.exe'
    & $venvPy -m pip install --upgrade pip -q
    & $venvPy -m pip install -q -r (Join-Path $legacyRoot 'requirements.txt')
    if ($LASTEXITCODE -ne 0) { throw 'mkdocs-material 安装失败，请检查网络后重试。' }
}

Write-Host '同步内容到旧站构建源 ...'
if (Test-Path $docs) { Remove-Item $docs -Recurse -Force }
New-Item -ItemType Directory -Path $docs | Out-Null

foreach ($directory in @('初试准备', '复试准备', '上岸经验分享', '毕业去向')) {
    Copy-Item (Join-Path $repoRoot $directory) $docs -Recurse
}
foreach ($file in @('经验分享投稿模板.md', 'CONTRIBUTORS.md', '免责声明.md')) {
    Copy-Item (Join-Path $repoRoot $file) $docs
}

& $venvPy (Join-Path $legacyRoot 'anonymize.py') $docs
Copy-Item (Join-Path $legacyRoot 'homepage.md') (Join-Path $docs 'index.md')

& $venvPy -m mkdocs build --config-file (Join-Path $legacyRoot 'mkdocs.yml')
if ($LASTEXITCODE -ne 0) { throw '旧站构建失败' }

if ($Serve) {
    Write-Host '启动旧站预览：http://127.0.0.1:8000 （Ctrl+C 停止）'
    & $venvPy -m http.server 8000 --directory $site --bind 127.0.0.1
} else {
    Write-Host "旧站构建完成：$(Join-Path $site 'index.html')"
}
