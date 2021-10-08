import React from 'react';

import Head from '@docusaurus/Head';
import { Redirect } from '@docusaurus/router';
import useBaseUrl from "@docusaurus/useBaseUrl";

const Home = () => {
    return <Redirect to={useBaseUrl('/getting_started/synth')} />;
};

export default Home;
