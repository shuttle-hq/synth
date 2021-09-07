const DB_CONTAINER_NAME = "message-board-example"

var shell = require('shelljs');

const main = async () => {
    if (!shell.which('docker')) {
        shell.echo('Could not find `docker` installed.')
    }

    const find = shell.exec(`docker ps -q -f "name=${DB_CONTAINER_NAME}"`)

    if (find.stdout === "") {
        const run = shell.exec(`docker run -d --name ${DB_CONTAINER_NAME} -it -p 27017:27017 --rm mongo`)
        if (run.code !== 0) {
            shell.echo(`Could not run docker container: ${run}`)
            shell.exit(run.code)
        }

        shell.echo(`Waiting for container to be ready`)
        await new Promise((resolve) => setTimeout(resolve, 5000))

        shell.echo(`Running 'synth generate'`)
        const synth = shell.exec(`synth generate --to mongodb://localhost:27017/board synth/ --size 1000`)
        if (synth.code !== 0) {
            shell.echo(`Could not run synth generate: ${synth}`)
        }
        shell.echo(`Done`)
        shell.exit(0)
    } else {
        shell.echo(`Container '${DB_CONTAINER_NAME}' already running, taking it down`)
        const rm = shell.exec(`docker rm -f ${DB_CONTAINER_NAME}`)
        if (rm.code !== 0) {
            shell.echo(`Could not remove docker container: ${rm}`)
            shell.exit(rm.code)
        }
        await main()
    }
}

main().catch((e) => {
    console.error(e)
    process.exit(1)
})