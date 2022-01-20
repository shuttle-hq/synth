import React from "react";
import CookieConsent from "../components/CookieConsent";

function Root({ children }) {
  return (
    <>
      {children}
      <CookieConsent />
    </>
  );
}

export default Root;
