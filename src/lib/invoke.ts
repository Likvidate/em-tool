import { invoke as rawInvoke } from "@tauri-apps/api/core";

export type CommandError = { code: string; message: string };

export class InvokeError extends Error {
  code: string;
  constructor(err: CommandError) {
    super(err.message);
    this.code = err.code;
  }
}

export async function invoke<T = void>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await rawInvoke<T>(command, args);
  } catch (err) {
    if (err && typeof err === "object" && "code" in err && "message" in err) {
      throw new InvokeError(err as CommandError);
    }
    throw err;
  }
}

export const vaultApi = {
  exists: () => invoke<boolean>("vault_exists"),
  isUnlocked: () => invoke<boolean>("is_unlocked"),
  create: (password: string) => invoke<void>("create_vault", { password }),
  unlock: (password: string) => invoke<void>("unlock_vault", { password }),
  lock: () => invoke<void>("lock_vault"),
  touchActivity: () => invoke<void>("touch_activity"),
};
