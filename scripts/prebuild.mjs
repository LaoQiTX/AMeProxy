import fs from 'fs'
import path from 'path'
import { fileURLToPath } from 'url'
import fetch from 'node-fetch'
import extract from 'extract-zip'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const SIDECAR_DIR = path.join(__dirname, '../src-tauri/sidecar')

// 平台映射
const PLATFORM_MAP = {
  'win32-x64': 'mihomo-windows-amd64-compatible',
  'win32-arm64': 'mihomo-windows-arm64',
  'win32-ia32': 'mihomo-windows-386',
  'darwin-x64': 'mihomo-darwin-amd64',
  'darwin-arm64': 'mihomo-darwin-arm64',
  'linux-x64': 'mihomo-linux-amd64',
  'linux-arm64': 'mihomo-linux-arm64'
}

// 默认版本号（当无法获取最新版本时使用）
const DEFAULT_VERSION = 'v1.19.3'

// 镜像源列表（优先级从高到低）
const MIRRORS = [
  {
    name: 'llkk',
    baseUrl: 'https://gh.llkk.cc/https://github.com/MetaCubeX/mihomo',
    versionUrl: 'https://gh.llkk.cc/https://github.com/MetaCubeX/mihomo/releases/latest/download/version.txt'
  },
  {
    name: 'bugdey',
    baseUrl: 'https://gh.bugdey.us.kg/https://github.com/MetaCubeX/mihomo/',
    versionUrl: 'https://gh.bugdey.us.kg/https://github.com/MetaCubeX/mihomo/releases/latest/download/version.txt'
  },
  {
    name: 'GitHub',
    baseUrl: 'https://github.com/MetaCubeX/mihomo',
    versionUrl: 'https://github.com/MetaCubeX/mihomo/releases/latest/download/version.txt'
  }
]

// 带超时的 fetch
async function fetchWithTimeout(url, options = {}, timeout = 30000) {
  const controller = new AbortController()
  const timeoutId = setTimeout(() => controller.abort(), timeout)
  
  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal
    })
    clearTimeout(timeoutId)
    return response
  } catch (error) {
    clearTimeout(timeoutId)
    throw error
  }
}

// 验证版本号格式（v1.2.3 或 v1.2.3-alpha）
function isValidVersion(version) {
  // 版本号应该以 v 开头，后面跟着数字
  return /^v\d+\.\d+\.\d+/.test(version)
}

// 尝试从镜像源获取版本
async function fetchVersion(mirror) {
  try {
    console.log(`[${mirror.name}] 尝试获取版本...`)
    const response = await fetchWithTimeout(mirror.versionUrl, {}, 10000)
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`)
    }
    const text = (await response.text()).trim()
    
    // 验证版本号格式
    if (!isValidVersion(text)) {
      throw new Error('返回的内容不是有效的版本号')
    }
    
    console.log(`[${mirror.name}] 成功获取版本: ${text}`)
    return text
  } catch (error) {
    console.log(`[${mirror.name}] 获取版本失败: ${error.message}`)
    return null
  }
}

// 检查文件是否为有效的 zip 文件
function isValidZip(filePath) {
  try {
    const buffer = fs.readFileSync(filePath)
    // ZIP 文件的魔数是 0x50 0x4B 0x03 0x04
    return buffer.length > 4 && 
           buffer[0] === 0x50 && 
           buffer[1] === 0x4B && 
           buffer[2] === 0x03 && 
           buffer[3] === 0x04
  } catch (error) {
    return false
  }
}

// 尝试从镜像源下载文件
async function downloadFile(mirror, version, targetFile) {
  // https://github.com/MetaCubeX/mihomo/releases/download/v1.19.22/mihomo-windows-amd64-compatible-v1.19.22.zip
  const downloadUrl = `${mirror.baseUrl}/releases/download/${version}/${targetFile}-${version}.zip`
  const zipPath = path.join(SIDECAR_DIR, `${targetFile}-${version}.zip`)
  
  try {
    console.log(`[${mirror.name}] 尝试下载内核...`)
    const response = await fetchWithTimeout(downloadUrl, {}, 60000)
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`)
    }
    
    const buffer = await response.arrayBuffer()
    fs.writeFileSync(zipPath, Buffer.from(buffer))
    
    // 验证下载的文件是否为有效的 zip
    if (!isValidZip(zipPath)) {
      fs.unlinkSync(zipPath)
      throw new Error('下载的文件不是有效的 ZIP 文件')
    }
    
    console.log(`[${mirror.name}] 下载成功`)
    return zipPath
  } catch (error) {
    console.log(`[${mirror.name}] 下载失败: ${error.message}`)
    // 清理可能存在的无效文件
    if (fs.existsSync(zipPath)) {
      try {
        fs.unlinkSync(zipPath)
      } catch (e) {}
    }
    return null
  }
}

// 下载内核
async function downloadMihomo() {
  const platform = `${process.platform}-${process.arch}`
  const targetFile = PLATFORM_MAP[platform]
  if (!targetFile) {
    throw new Error(`Unsupported platform: ${platform}`)
  }

  console.log(`平台: ${platform}`)
  console.log(`目标文件: ${targetFile}`)
  console.log('')

  // 创建目录
  if (!fs.existsSync(SIDECAR_DIR)) {
    fs.mkdirSync(SIDECAR_DIR, { recursive: true })
  }

  let version = null
  let workingMirror = null

  // 尝试从各个镜像源获取版本
  for (const mirror of MIRRORS) {
    version = await fetchVersion(mirror)
    if (version) {
      workingMirror = mirror
      break
    }
  }

  if (!version) {
    console.log('所有镜像源都无法获取版本信息，使用默认版本')
    version = DEFAULT_VERSION
    workingMirror = MIRRORS[0] // 使用第一个镜像源
  }

  console.log(`使用镜像源: ${workingMirror ? workingMirror.name : '默认'}`)
  console.log(`版本: ${version}`)
  console.log('')

  let zipPath = null

  // 尝试从各个镜像源下载（从成功的镜像源开始）
  const mirrorIndex = MIRRORS.indexOf(workingMirror)
  const mirrorsToTry = [
    workingMirror,
    ...MIRRORS.slice(mirrorIndex + 1),
    ...MIRRORS.slice(0, mirrorIndex)
  ]

  for (const mirror of mirrorsToTry) {
    zipPath = await downloadFile(mirror, version, targetFile)
    if (zipPath) {
      workingMirror = mirror
      break
    }
  }

  if (!zipPath) {
    throw new Error('所有镜像源都无法下载内核')
  }

  console.log(`使用镜像源下载: ${workingMirror.name}`)

  // 解压
  try {
    console.log('解压中...')
    await extract(zipPath, { dir: SIDECAR_DIR })
    console.log('解压成功')
  } catch (error) {
    throw new Error(`解压失败: ${error.message}`)
  }

  // 列出解压后的目录内容
  try {
    console.log('解压后目录内容:')
    const entries = fs.readdirSync(SIDECAR_DIR)
    entries.forEach(entry => {
      const entryPath = path.join(SIDECAR_DIR, entry)
      const stat = fs.statSync(entryPath)
      console.log(`  ${entry} (${stat.isDirectory() ? '目录' : '文件'})`)
    })
  } catch (error) {
    console.warn('列出目录内容失败:', error.message)
  }

  // 重命名 - Tauri 2.0+ 需要特定的命名格式: {name}-{target-triple}.{extension}
  const targetTriple = process.platform === 'win32' ? 'x86_64-pc-windows-msvc' : 
                     process.platform === 'darwin' ? 'aarch64-apple-darwin' : 'x86_64-unknown-linux-gnu'
  const exePath = path.join(SIDECAR_DIR, `my-mihomo-${targetTriple}${process.platform === 'win32' ? '.exe' : ''}`)
  
  // 查找 mihomo 可执行文件
  let mihomoPath = null
  try {
    const entries = fs.readdirSync(SIDECAR_DIR)
    
    // 直接查找 mihomo 可执行文件
    mihomoPath = entries.find(entry => {
      const entryPath = path.join(SIDECAR_DIR, entry)
      return fs.statSync(entryPath).isFile() && 
             (entry === 'mihomo.exe' || entry === 'mihomo' || entry.includes('mihomo') && (entry.endsWith('.exe') || entry.endsWith('.bin')))
    })
    
    if (mihomoPath) {
      mihomoPath = path.join(SIDECAR_DIR, mihomoPath)
      console.log(`找到可执行文件: ${mihomoPath}`)
    } else {
      // 如果直接找不到，查找目录中的文件
      const dirs = entries.filter(entry => {
        const entryPath = path.join(SIDECAR_DIR, entry)
        return fs.statSync(entryPath).isDirectory()
      })
      
      for (const dir of dirs) {
        const dirPath = path.join(SIDECAR_DIR, dir)
        const dirEntries = fs.readdirSync(dirPath)
        const exeFile = dirEntries.find(entry => 
          entry === 'mihomo.exe' || entry === 'mihomo'
        )
        if (exeFile) {
          mihomoPath = path.join(dirPath, exeFile)
          console.log(`在目录 ${dir} 中找到可执行文件: ${mihomoPath}`)
          break
        }
      }
    }
    
    if (!mihomoPath) {
      throw new Error('找不到 mihomo 可执行文件')
    }
  } catch (error) {
    throw new Error(`查找可执行文件失败: ${error.message}`)
  }
  
  // 重命名
  if (fs.existsSync(mihomoPath)) {
    fs.renameSync(mihomoPath, exePath)
    console.log('重命名成功')
  } else {
    throw new Error(`找不到解压后的文件: ${mihomoPath}`)
  }

  // 清理
  try {
    fs.unlinkSync(zipPath)
    // 清理其他临时文件和目录
    const entries = fs.readdirSync(SIDECAR_DIR)
    const expectedFileName = `my-mihomo-${targetTriple}${process.platform === 'win32' ? '.exe' : ''}`
    entries.forEach(entry => {
      const entryPath = path.join(SIDECAR_DIR, entry)
      if (entry !== expectedFileName) {
        try {
          const stat = fs.statSync(entryPath)
          if (stat.isDirectory()) {
            fs.rmSync(entryPath, { recursive: true, force: true })
          } else {
            fs.unlinkSync(entryPath)
          }
        } catch (e) {
          console.warn(`清理 ${entry} 失败:`, e.message)
        }
      }
    })
    console.log('清理临时文件成功')
  } catch (error) {
    console.warn('清理临时文件失败:', error.message)
  }

  console.log('')
  console.log('✓ Mihomo 下载成功!')
}

// 执行
downloadMihomo().catch((error) => {
  console.error('')
  console.error('✗ 下载失败:', error.message)
  console.log('')
  console.log('提示: 您可以手动下载 mihomo 内核并放置到 src-tauri/sidecar/ 目录')
  console.log('下载地址: https://github.com/MetaCubeX/mihomo/releases')
  console.log('')
  console.log('Continuing with build process...')
})