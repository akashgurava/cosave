#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');

// Dynamically locate frontend/node_modules/@playwright/test
function getPlaywright() {
  const candidates = [
    path.resolve(__dirname, '../../../../frontend/node_modules/@playwright/test'),
    path.resolve(process.cwd(), 'frontend/node_modules/@playwright/test'),
    path.resolve(process.cwd(), 'node_modules/@playwright/test')
  ];
  for (const c of candidates) {
    if (fs.existsSync(c)) {
      return require(c);
    }
  }
  return require('@playwright/test');
}

const { chromium } = getPlaywright();

const targetUrl = process.argv[2] || 'http://localhost:5172';
const outputDir = process.argv[3] || path.resolve(process.cwd(), '.scratch/ui-review');

if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

async function capture() {
  const browser = await chromium.launch();
  try {
    // 1. Desktop Dark
    const desktopContext = await browser.newContext({
      viewport: { width: 1280, height: 800 },
      colorScheme: 'dark'
    });
    const desktopPage = await desktopContext.newPage();
    await desktopPage.goto(targetUrl, { waitUntil: 'networkidle', timeout: 15000 });
    
    // Ensure dark theme
    await desktopPage.evaluate(() => {
      document.documentElement.setAttribute('data-theme', 'dark');
      document.documentElement.classList.add('dark');
    });
    await desktopPage.waitForTimeout(300);
    const darkPath = path.join(outputDir, 'desktop-dark.png');
    await desktopPage.screenshot({ path: darkPath, fullPage: true });

    // 2. Desktop Light
    await desktopPage.evaluate(() => {
      localStorage.setItem('cosave-theme', 'light');
      document.documentElement.setAttribute('data-theme', 'light');
      document.documentElement.classList.remove('dark');
    });
    await desktopPage.reload({ waitUntil: 'networkidle' });
    await desktopPage.waitForTimeout(300);
    const lightPath = path.join(outputDir, 'desktop-light.png');
    await desktopPage.screenshot({ path: lightPath, fullPage: true });
    await desktopContext.close();

    // 3. Mobile Dark
    const mobileContext = await browser.newContext({
      viewport: { width: 390, height: 844 },
      isMobile: true,
      colorScheme: 'dark'
    });
    const mobilePage = await mobileContext.newPage();
    await mobilePage.goto(targetUrl, { waitUntil: 'networkidle', timeout: 15000 });
    await mobilePage.evaluate(() => {
      document.documentElement.setAttribute('data-theme', 'dark');
      document.documentElement.classList.add('dark');
    });
    await mobilePage.waitForTimeout(300);
    const mobilePath = path.join(outputDir, 'mobile-dark.png');
    await mobilePage.screenshot({ path: mobilePath, fullPage: true });
    await mobileContext.close();

    const results = {
      url: targetUrl,
      outputDir,
      screenshots: {
        desktopDark: darkPath,
        desktopLight: lightPath,
        mobileDark: mobilePath
      }
    };

    console.log(JSON.stringify(results, null, 2));
  } finally {
    await browser.close();
  }
}

capture().catch((err) => {
  console.error('Failed to capture screenshots:', err);
  process.exit(1);
});
