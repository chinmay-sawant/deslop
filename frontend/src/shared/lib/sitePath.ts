const siteBase = import.meta.env.BASE_URL

export function sitePath(path = '') {
  return `${siteBase}${path.replace(/^\//, '')}`
}

export function isHomePath(pathname: string) {
  const homePath = siteBase.endsWith('/') ? siteBase.slice(0, -1) : siteBase

  return pathname === '' || pathname === '/' || pathname === homePath || pathname === `${homePath}/`
}