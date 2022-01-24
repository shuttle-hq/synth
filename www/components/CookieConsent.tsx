import { useEffect, useState } from "react"
import Link from "next/link"
import { getCookie, setCookie } from "../lib/helpers"

const CookieConsent = () => {
  const COOKIE_CONSENT_NAME = "COOKIE_CONSENT_NAME"
  const [visible, setVisible] = useState(false)

  useEffect(() => {
    if (getCookie(COOKIE_CONSENT_NAME) === "true") {
      setVisible(false)
    } else {
      setVisible(true)
    }
  }, [visible])

  const handleAccept = () => {
    setCookie(COOKIE_CONSENT_NAME, "true", 7)
    setVisible(false)
  }

  return (
    visible && (
      <div className="sticky left-0 bottom-0 w-full z-50 bg-dark-600 border-t border-dark-500">
        <div className="container max-w-4xl mx-auto">
          <div className="flex flex-wrap items-center py-3 gap-5 text-sm px-10 xl:px-20">
            <div className="w-full xl:flex-1 xl:w-auto">
              We use cookies to enhance your experience, analyze our traffic,
              and for security and marketing. By visiting our website you agree
              to our use of cookies.{" "}
              <strong>
                <Link href="/privacy">
                  <a className="text-accent-1">*Read more about cookies*</a>
                </Link>
              </strong>
            </div>
            <div className="w-full xl:w-auto">
              <button
                className="text-white bg-brand-600 hover:bg-brand-400 font-bold py-2 px-3 text-sm focus:outline-none relative inline-flex items-center rounded border-transparent transition"
                onClick={handleAccept}
              >
                I Accept
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  )
}

export default CookieConsent
