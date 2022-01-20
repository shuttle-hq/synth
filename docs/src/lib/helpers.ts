export function setCookie(name: string, value: string, days: number) {
  let expires = ""

  if (days) {
    const date = new Date()
    date.setTime(date.getTime() + days * 24 * 60 * 60 * 1000)
    expires = "; expires=" + date.toUTCString()
  }

  document.cookie =
    name + "=" + (value || "") + expires + "; Path=/; SameSite=Strict;"
}

export function getCookie(name: string) {
  const nameEQ = name + "="
  const ca = document.cookie.split(";")

  for (let i = 0; i < ca.length; i++) {
    let c = ca[i]
    while (c.charAt(0) === " ") {
      c = c.substring(1, c.length)
    }
    if (c.indexOf(nameEQ) === 0) {
      return c.substring(nameEQ.length, c.length)
    }
  }

  return undefined
}

export function removeCookie(name: string) {
  document.cookie =
    name + "=; Path=/; SameSite=Strict; Expires=Thu, 01 Jan 1970 00:00:01 GMT;"
}
