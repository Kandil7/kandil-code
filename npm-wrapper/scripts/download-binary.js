const https = require('https')
const fs = require('fs')
const path = require('path')
const crypto = require('crypto')
const HttpsProxyAgent = require('https-proxy-agent')

const map = {
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'win32-x64': 'x86_64-pc-windows-msvc'
}

const key = `${process.platform}-${process.arch}`
const target = map[key]
if (!target) { console.error('Unsupported platform'); process.exit(1) }

const isWin = process.platform === 'win32'
const asset = isWin ? `kandil-${target}.zip` : `kandil-${target}.tar.gz`
const sha = `kandil-${target}.sha256`
const base = 'https://github.com/Kandil7/kandil_code/releases/latest/download'
const url = `${base}/${asset}`
const shaUrl = `${base}/${sha}`
const outDir = path.join(__dirname, '..', 'bin')
if (!fs.existsSync(outDir)) fs.mkdirSync(outDir, { recursive: true })

const archive = path.join(outDir, asset)
const agent = process.env.HTTPS_PROXY ? new HttpsProxyAgent.HttpsProxyAgent(process.env.HTTPS_PROXY) : undefined

function fetch(u, file) {
  return new Promise((resolve, reject) => {
    const opts = agent ? { agent } : {}
    https.get(u, opts, res => {
      if (res.statusCode !== 200) return reject(new Error('download failed'))
      const f = fs.createWriteStream(file)
      res.pipe(f)
      f.on('finish', () => { f.close(resolve) })
    }).on('error', reject)
  })
}

function verify(arch, shaFile) {
  const expected = fs.readFileSync(shaFile, 'utf8').trim()
  const hash = crypto.createHash('sha256')
  const data = fs.readFileSync(arch)
  hash.update(data)
  const actual = hash.digest('hex')
  if (expected.toLowerCase() !== actual.toLowerCase()) throw new Error('checksum mismatch')
}

async function main() {
  const shaPath = path.join(outDir, sha)
  await fetch(url, archive)
  await fetch(shaUrl, shaPath)
  verify(archive, shaPath)
  if (isWin) {
    const unzip = require('unzipper')
    await new Promise((resolve, reject) => {
      fs.createReadStream(archive).pipe(unzip.Extract({ path: outDir })).on('close', resolve).on('error', reject)
    })
  } else {
    const tar = require('tar')
    await tar.x({ file: archive, cwd: outDir })
    const dest = path.join(outDir, 'kandil')
    fs.chmodSync(dest, 0o755)
  }
}

main().catch(e => { console.error(String(e)); process.exit(1) })
