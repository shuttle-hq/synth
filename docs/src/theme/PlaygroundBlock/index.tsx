import React, {useState, useEffect} from 'react';

import CodeBlock from '@theme/CodeBlock'

import {PlaygroundError, pgGenerate} from '../../lib/playground';

type Querying = {
    step: "querying"
};

const Querying: Querying = {step: "querying"};

type Failed = {
    step: "failed",
    error: PlaygroundError
};

const Failed = (error: PlaygroundError): Failed => {
    return {
        step: "failed",
        error
    }
}

type Ok = {
    step: "ok",
    generated: any
};

const Ok = (generated: any): Ok => {
    return {
        step: "ok",
        generated
    }
}

type PlaygroundState = Querying | Failed | Ok;

type PlaygroundProps = {
    schema: any,
    size?: number,
    seed: number
}

const PlaygroundBlock = ({schema, size, seed}: PlaygroundProps) => {
    let [state, setState] = useState<PlaygroundState>(Querying);
    let [seedState, setSeedState] = useState<number | null>(null);

    useEffect(() => {
        if (seedState != seed) {
            setSeedState(seed);
            setState(Querying);
        }
        if (state.step === "querying") {
            const baseUrl = process.env.NODE_ENV === "development"
                ? "http://localhost:8182"
                : "https://dev.getsynth.com";
            pgGenerate(schema, size, baseUrl)
                .then((generated) => {
                    setState(Ok(generated));
                })
                .catch((err: PlaygroundError) => {
                    setState(Failed(err));
                })
        }
    });

    return (
        <CodeBlock className="language-json" metastring="" isResult={true}>
            {
                state.step == "querying"
                && 'Generating...'
                || (
                    state.step == "ok"
                    && JSON.stringify(state.generated, null, 2)
                ) || (
                    state.step == "failed"
                    && `${state.error}`
                )
            }
        </CodeBlock>
    );
}

export default PlaygroundBlock;