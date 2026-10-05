import { api } from "./client";
export const importWallpaper = (path: string): Promise<string> => api.invoke("wallpaper_import", { path });
export const prepareWallpaperVideo = (path: string, width: number, fps: number): Promise<string> => api.invoke("wallpaper_prepare_video", { path, width, fps });
export const prepareWallpaperPoster = (path: string): Promise<string> => api.invoke("wallpaper_prepare_poster", { path });
