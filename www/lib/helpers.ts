import type { NextRouter } from "next/router"

import * as Fathom from "fathom-client"

import posthog from "posthog-js"

import { SITE_DOMAINS, FATHOM_API_KEY, POSTHOG_API_KEY } from "./constants"

export function setupFathom(router: NextRouter) {
  if (process.env.NODE_ENV === "development") {
    console.warn("Fathom telemetry inhibited due to dev environment")
    return
  }

  Fathom.load(FATHOM_API_KEY, {
    includedDomains: SITE_DOMAINS,
  })

  function onRouteChangeComplete() {
    Fathom.trackPageview()
  }

  router.events.on("routeChangeComplete", onRouteChangeComplete)

  return () => {
    router.events.off("routeChangeComplete", onRouteChangeComplete)
  }
}

export function setupPostHog() {
  posthog.init(POSTHOG_API_KEY, {
    api_host: "https://app.posthog.com",
  })
  if (process.env.NODE_ENV === "development") {
    console.warn("PostHog capturing inhibited due to dev environment")
    posthog.debug()
    posthog.opt_out_capturing()
  }
}

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
