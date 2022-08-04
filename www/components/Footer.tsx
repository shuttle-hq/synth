import {FontAwesomeIcon} from "@fortawesome/react-fontawesome";

import {useRouter} from 'next/router'

import {faGithub, faTwitter, faDiscord, faLinkedin} from "@fortawesome/free-brands-svg-icons";

import {ChatWidget} from "@papercups-io/chat-widget";

import Link from 'next/link'

import Logo from "./Logo";

const Footer = () => {
    const {basePath} = useRouter();
    return (
        <div className="relative w-full bg-gray-600">
            <div className="container w-10/12 xl:w-8/12 xl:px-12 py-5 mx-auto">
                <div className="pt-16 pb-16 grid grid-cols-1 md:grid-cols-4 lg:grid-cols-6">
                    <div className="col-span-2 md:col-span-4 lg:col-span-2">
                        <Logo classNameLarge='h-16'/>
                        <div className="flex flex-row">
                            <div className="pt-4 pb-3 grid gap-y-4 grid-rows-1 grid-cols-4">
                                <a target= "_blank" className = "pr-4" href="https://github.com/getsynth/synth">
                                    <FontAwesomeIcon className="h-8 hover:text-white transition" icon={faGithub}/>
                                </a>
                            </div>
                        </div>
                    </div>
                    <div>
                        <div className="grid text-dark-300 font-medium lg:grid-rows-4 gap-4 py-4">
                            <div className="text-dark-400 font-semibold font-mono uppercase">
                                Product
                            </div>
                            <div>
                                <Link href="/#use-cases">Use cases</Link>
                            </div>
                            <div>
                                <Link href="/#features">Features</Link>
                            </div>
                            <div>
                                <Link href="/#snippets">Examples</Link>
                            </div>
                            <div>
                                <Link href="/download">Download</Link>
                            </div>
                        </div>
                    </div>
                    <div>
                        <div className="grid text-dark-300 font-medium grid-rows-4 gap-4 py-4">
                            <div className="text-dark-400 font-semibold font-mono uppercase">
                                Learn
                            </div>
                            <div>
                                <Link href="/docs/getting_started/hello-world">Getting Started</Link>
                            </div>
                            <div>
                                <Link href="/docs/content/index">API Reference</Link>
                            </div>
                            <div>
                                <Link href="/docs/examples/bank">Examples</Link>
                            </div>
                        </div>
                    </div>
                    <div>
                        <div className="grid text-dark-300 font-medium grid-rows-2 gap-4 py-4">
                            <div className="text-dark-400 font-semibold font-mono uppercase">
                                Community
                            </div>
                            <div>
                                <Link href="https://github.com/getsynth/synth">Github</Link>
                            </div>
                        </div>
                    </div>
                    <div>
                        <div className="grid text-dark-300 font-medium grid-rows-4 gap-4 py-4">
                            <div className="text-dark-400 font-semibold font-mono uppercase">
                                More
                            </div>
                            <div>
                                <Link href="/contact">Contact</Link>
                            </div>
                            <div>
                                <Link href="/terms">T&Cs</Link>
                            </div>
                            <div>
                                <Link href="/privacy">Privacy Policy</Link>
                            </div>
                        </div>
                    </div>
                </div>
                <div className=" border-t border-gray-400 pt-4"/>
                <div className="pb-16 text-sm text-gray-300">
                    &copy; 2021 OpenQuery Inc.
                </div>
            </div>
        </div>
    )
}

export default Footer