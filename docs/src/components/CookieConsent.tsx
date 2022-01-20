import React from "react";
import { useEffect, useState } from "react";
import { getCookie, setCookie } from "../lib/helpers";

const CookieConsent = () => {
  const COOKIE_CONSENT_NAME = "COOKIE_CONSENT_NAME";
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (getCookie(COOKIE_CONSENT_NAME) === "true") {
      setVisible(false);
    } else {
      setVisible(true);
    }
  }, [visible]);

  const handleAccept = () => {
    setCookie(COOKIE_CONSENT_NAME, "true", 7);
    setVisible(false);
  };

  return (
    visible && (
      <div className="cookie-consent">
        <div className="container">
          <div className="cookie-consent-wrapper">
            <div className="cookie-consent-info">
              We use cookies to enhance your experience, analyze our traffic,
              and for security and marketing. By visiting our website you agree
              to our use of cookies.{" "}
              <strong>
                <a href="/privacy">
                  *Read more about cookies*
                </a>
              </strong>
            </div>
            <div className="cookie-consent-action">
              <button
                className="cookie-consent-button"
                onClick={handleAccept}
              >
                I Accept
              </button>
            </div>
          </div>
        </div>
      </div>
    )
  );
};

export default CookieConsent;
