import {PrismaClient} from '@prisma/client'

const prisma = new PrismaClient()

const main = async () => {
    const user = await prisma.user.findFirst()
    if (user === null) {
        throw Error("No user data.")
    }

    const posts: { title: string, postedAt: Date }[] = await prisma.post.findMany({
        "where": {
            "authorId": user.id
        }
    })
    if (posts.length === 0) {
        throw Error("User has no post.")
    }

    console.log(
        `
Data about first user:
        username: ${user.nickname}
        email: ${user.email}
        num posts: ${posts.length}
        first post title: ${posts[0].title}
        first posted at:  ${posts[0].postedAt}`
    )

    process.exit(0)
}

main().catch((e) => {
    console.error(e)
    process.exit(1)
})