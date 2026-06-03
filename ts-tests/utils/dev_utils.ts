/** Seals `count` empty blocks via manual seal — used to age out a rate-limit window. */
export async function seal(context: DevModeContext, count: number): Promise<void> {
    for (let i = 0; i < count; i++) {
        await context.createBlock();
    }
}
