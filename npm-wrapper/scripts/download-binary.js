const https = require('https')
const fs = require('fs')
const path = require('path')
const os = require('os')

const map = {
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'win32-x64': 'x86_64-pc-windows-msvc'
}

const key = `${process.platform}-${process.arch}`
const target = map[key]
if (!target) {
  console.error('Unsupported platform')
  process.exit(1)
}

const isWin = process.platform === 'win32'
const asset = isWin ? `kandil-${target}.zip` : `kandil-${target}.tar.gz`
const url = `https://github.com/Kandil7/kandil_code/releases/latest/download/${asset}`
const outDir = path.join(__dirname, '..', 'bin')
if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true })

const dest = path.join(outDir, isWin ? 'kandil.exe' : 'kandil')

function download(u, file, cb) {
  const f = fs.createWriteStream(file)
  https.get(u, res => {
    if (res.statusCode !== 200) { console.error('Download failed'); process.exit(1) }
    res.pipe(f)
    f.on('finish', () => { f.close(cb) })
  }).on('error', err => { console.error(err); process.exit(1) })
}

function extract(archive, cb) {
  if (isWin) {
    const unzip = require('unzipper')
    fs.createReadStream(archive)
      .pipe(unzip.Extract({ path: outDir }))
      .on('close', cb)
  } else {
    const { spawn } = require('child_process')
    const tar = spawn('tar', ['xzf', archive, '-C', outDir])
    tar.on('exit', code => { if (code === 0) cb(); else { console.error('Extract failed'); process.exit(1) } })
  }
}

const archive = path.join(outDir, asset)
download(url, archive, () => {
  extract(archive, () => {
    if (!isWin) fs.chmodSync(dest, 0o755)
  })
})
