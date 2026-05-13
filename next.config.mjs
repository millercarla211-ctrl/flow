/** @type {import("next").NextConfig} */
const isVercel = process.env.VERCEL === "1";

const nextConfig = {
  ...(isVercel ? {} : { output: "export" }),
  images: {
    unoptimized: true,
  },
  reactStrictMode: true,
  trailingSlash: true,
  devIndicators: false,
};

export default nextConfig;
