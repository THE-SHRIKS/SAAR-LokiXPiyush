# Pixel Perfect Replica

Implement exactly the screenshot and nothing else

This project was built with [Lovable](https://lovable.dev).

## Build with Lovable

Continue developing this project in the [Lovable editor](https://lovable.dev/projects/e11886b0-01cd-48df-85f9-179fc098d0f2).

- **Ship faster**: describe what you want to build and Lovable handles the code.
- **Stay in sync**: every change made in Lovable is committed straight to this repository.
- **Full ownership**: this code is yours. Push to `main` on GitHub and your changes sync back into Lovable, ready for your next prompt.

## Development

Prefer working locally? You need Node.js and npm — [install with nvm](https://github.com/nvm-sh/nvm#installing-and-updating).

```sh
git clone <this-repository-url>
cd <repository-name>
npm i
npm run dev
```

## Deploy to Vercel

1. Import this folder (`SAAR-Website/web`) as a Vercel project. `vercel.json` sets the install and build commands and the Vercel Nitro preset.
2. In the Vercel project settings, add these environment variables for Production:
   - `VITE_SITE_URL`: the production origin, for example `https://your-domain.example`. It sets canonical links, Open Graph URLs, the sitemap, and `robots.txt`.
   - `VITE_DOWNLOAD_URL` (optional): where the Windows installer is hosted. Leave it unset to serve `public/downloads/SAAR-Setup.exe` from the site.
3. The installer is not committed. The Windows build is about 210 MiB because it bundles the offline WebView2 runtime, which is larger than Vercel's static file limit. Host `SAAR-Setup.exe` somewhere that accepts large files, such as a GitHub Release, and set `VITE_DOWNLOAD_URL` to its direct download link. For local testing, copy `SAAR-Setup.exe` and `SAAR-Setup.exe.sha256` into `public/downloads/`; that folder is ignored by Git.

Build locally with `npm run build`. The `prebuild` step writes `public/sitemap.xml` and `public/robots.txt` from `VITE_SITE_URL`.
