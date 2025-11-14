const { spawn } = require('child_process')
const path = require('path')
const fs = require('fs')

const bin = process.platform === 'win32' ? 'kandil.exe' : 'kandil'
const binaryPath = path.join(__dirname, '..', 'bin', bin)

if (!fs.existsSync(binaryPath)) {
  console.error('Kandil binary not found')
  process.exit(1)
}

const child = spawn(binaryPath, process.argv.slice(2), { stdio: 'inherit' })
child.on('exit', code => process.exit(code))
