import AccentButton from "./AccentButton";
import {faExternalLinkAlt} from "@fortawesome/free-solid-svg-icons";
import {useRouter} from "next/router";

type CallToActionProps = {
    copy?: string
}

const CallToAction = ({copy}: CallToActionProps) => {
    const {basePath} = useRouter();
    copy = copy == null ? "Synth helps you write better software, faster!" : copy;
    return (
        <div className="relative w-full bg-gray-700">
            <div className="container lg:w-8/12 w-10/12 mx-auto">
                <div className="text-center pt-32 pb-28">
                    <div className="text-2xl pb-3">
                        {copy}
                    </div>
                </div>
            </div>
        </div>
    )
}

export default CallToAction
