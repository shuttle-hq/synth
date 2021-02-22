import React, {useState, useEffect} from 'react';
import CodeBlock from '../CodeBlock'

type ErrorResponse = {
    status: number,
    kind?: string,
    text?: string
}

class PlaygroundError extends Error {
    response: ErrorResponse;

    constructor(response: ErrorResponse) {
        super(`status=${response.status} kind=${response.kind} text=${response.text}`);
        this.response = response;
    }
}

const pgGenerate = async function (
    req: any,
    size: number | null = null,
    baseUrl: string = "https://dev.getsynth.com"
): Promise<any> {
    const params = {
        method: "PUT",
        body: req,
        headers: {
            "Content-Type": "application/json"
        }
    };
    const query = size === null ? "" : `?size=${size}`;
    const url = `${baseUrl}/playground${query}`;
    return fetch(url, params)
        .then((response) => {
            if (response.status != 200) {
                if (response.headers.get("Content-Type") == "application/json") {
                    return response
                        .json()
                        .then((err) => {
                            throw new PlaygroundError({
                                status: response.status,
                                kind: err["kind"],
                                text: err["text"]
                            })
                        })
                } else {
                    throw new PlaygroundError({status: response.status})
                }
            } else {
                return response.json();
            }
        })
}

export {ErrorResponse, PlaygroundError, pgGenerate};

type PlaygroundState =
    { step: "querying" } |
    { step: "failed", response?: ErrorResponse } |
    { step: "ok", generated: any };

type PlaygroundProps = {
    schema: any,
    size?: number,
    seed: number
}

const PlaygroundBlock = ({ schema, size, seed }: PlaygroundProps) => {
    let [state,setState] = useState<PlaygroundState>({step: "querying"});
    let [seedState, setSeedState] = useState<number | null>(null);

    useEffect(() => {
        if (seedState != seed) {
            setSeedState(seed);
            setState({step: "querying"});
        }
        if (state.step == "querying") {
            const baseUrl = process.env.NODE_ENV === "development"
                ? "http://localhost:8182"
                : "https://dev.getsynth.com";
            pgGenerate(schema, size, baseUrl)
                .then((generated) => {
                    setState({ step: "ok", generated });
                })
                .catch((err: PlaygroundError) => {
                    setState({ step: "failed", response: err.response });
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
                    && `Error with ${state.response.kind}: ${state.response.text}`
                )
            }
        </CodeBlock>
    );
}

export default PlaygroundBlock;