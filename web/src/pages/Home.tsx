import { Link } from 'react-router-dom';
import { Navbar } from '../components/layout/Navbar';
import { Footer } from '../components/layout/Footer';
import { Logo } from '../components/ui';

/* ------------------------------------------------------------------ */
/*  Inline SVG icons                                                   */
/* ------------------------------------------------------------------ */

function IconArrowRight() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="5" x2="19" y1="12" y2="12" />
      <polyline points="12 5 19 12 12 19" />
    </svg>
  );
}

function IconDownload() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
      <polyline points="7 10 12 15 17 10" />
      <line x1="12" x2="12" y1="15" y2="3" />
    </svg>
  );
}

function IconCode() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="16 18 22 12 16 6" />
      <polyline points="8 6 2 12 8 18" />
    </svg>
  );
}

function IconBrain() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M9.5 2A2.5 2.5 0 0 1 12 4.5v15a2.5 2.5 0 0 1-4.96.44A2.5 2.5 0 0 1 4.5 17.5a2.5 2.5 0 0 1-.44-4.96A2.5 2.5 0 0 1 7 9.5a2.5 2.5 0 0 1 2.5-2.5V2z" />
      <path d="M14.5 2A2.5 2.5 0 0 0 12 4.5v15a2.5 2.5 0 0 0 4.96.44A2.5 2.5 0 0 0 19.5 17.5a2.5 2.5 0 0 0 .44-4.96A2.5 2.5 0 0 0 17 9.5a2.5 2.5 0 0 0-2.5-2.5V2z" />
    </svg>
  );
}

function IconPuzzle() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M19.439 7.85c-.049.322.059.648.289.878l1.568 1.568c.47.47.706 1.087.706 1.704s-.235 1.233-.706 1.704l-1.611 1.611a.98.98 0 0 1-.837.276c-.47-.07-.802-.48-.968-.925a2.501 2.501 0 1 0-3.214 3.214c.446.166.855.497.925.968a.979.979 0 0 1-.276.837l-1.61 1.61a2.404 2.404 0 0 1-1.705.707 2.402 2.402 0 0 1-1.704-.706l-1.568-1.568a1.026 1.026 0 0 0-.878-.29c-.493.074-.84.504-1.02.968a2.5 2.5 0 1 1-3.237-3.237c.464-.18.894-.527.967-1.02a1.026 1.026 0 0 0-.289-.878l-1.568-1.568A2.402 2.402 0 0 1 1.998 12c0-.617.236-1.234.706-1.704L4.315 8.685a.98.98 0 0 1 .837-.276c.47.07.802.48.968.925a2.501 2.501 0 1 0 3.214-3.214c-.446-.166-.855-.497-.925-.968a.979.979 0 0 1 .276-.837l1.61-1.61a2.404 2.404 0 0 1 1.705-.707c.618 0 1.234.236 1.704.706l1.568 1.568c.23.23.556.338.878.29.493-.074.84-.504 1.02-.968a2.5 2.5 0 1 1 3.237 3.237c-.464.18-.894.527-.967 1.02Z" />
    </svg>
  );
}

function IconPalette() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="13.5" cy="6.5" r="0.5" fill="currentColor" />
      <circle cx="17.5" cy="10.5" r="0.5" fill="currentColor" />
      <circle cx="8.5" cy="7.5" r="0.5" fill="currentColor" />
      <circle cx="6.5" cy="12.5" r="0.5" fill="currentColor" />
      <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.555C21.965 6.012 17.461 2 12 2z" />
    </svg>
  );
}

function IconSpeed() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2" />
    </svg>
  );
}

function IconShield() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
      <path d="m9 12 2 2 4-4" />
    </svg>
  );
}

function IconApple() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
      <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39c.87-1.52 2.43-2.48 4.12-2.51 1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z"/>
    </svg>
  );
}

function IconWindows() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
      <path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4H10.949M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-12.9-1.801"/>
    </svg>
  );
}

function IconLinux() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="currentColor">
      <path d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139zm.529 3.405h.013c.213 0 .396.062.584.198.19.135.33.332.438.533.105.259.158.459.166.724 0-.02.006-.04.006-.06v.105a.086.086 0 01-.004-.021l-.004-.024a1.807 1.807 0 01-.15.706.953.953 0 01-.213.335.71.71 0 00-.088-.042c-.104-.045-.198-.064-.284-.133a1.312 1.312 0 00-.22-.066c.05-.06.146-.133.183-.198.053-.128.082-.264.088-.402v-.02a1.21 1.21 0 00-.061-.4c-.045-.134-.101-.2-.183-.333-.084-.066-.167-.132-.267-.132h-.016c-.093 0-.176.03-.262.132a.8.8 0 00-.205.334 1.18 1.18 0 00-.09.468v.024c.016.066.032.132.057.2.056.132.14.2.245.264.094.066.203.132.29.132h.016a.466.466 0 00.238-.066c.024-.01.038-.018.053-.024v.004l.019-.012.075-.05c.088.198.455.598.666.733.042.033.09.062.135.094a1.075 1.075 0 00-.125.066c-.277.2-.47.532-.534.869-.065.337-.038.738.137 1.069.135.267.357.532.602.732v.003c.098.063.196.135.262.2.087.083.086.066.086.2v.266c0 .2-.013.4-.055.532-.042.198-.1.332-.189.533a2.071 2.071 0 01-.239.4c-.073.064-.159.132-.209.198h-.005a.43.43 0 00-.102.132.467.467 0 00-.049.2c0 .066.003.132.017.198l.005.003c.09.388.294.653.617.866.369.2.772.266 1.098.333.327.066.6.2.741.4.066.133.098.2.098.4 0 .066-.005.133-.022.2-.116.2-.27.4-.415.533-.145.132-.282.264-.373.463-.022.133.005.2.05.265a.39.39 0 00.183.135c.074.033.159.066.247.066h.016c.099 0 .202-.017.313-.066.109-.033.205-.066.287-.132a.448.448 0 00.163-.2.438.438 0 00.046-.2v-.003c0-.066-.002-.133-.022-.199-.042-.133-.1-.2-.165-.333h-.002c-.115-.132-.253-.2-.403-.266-.047-.021-.1-.032-.15-.049-.071-.033-.137-.07-.2-.135l.003.003-.008-.005a.31.31 0 01-.003-.2c.039-.066.106-.132.181-.199.149-.132.303-.264.429-.463.125-.2.22-.466.274-.732a2.49 2.49 0 00.046-.466v-.133c.016-.2.016-.4-.006-.6-.005-.133-.019-.264-.042-.332a1.076 1.076 0 00-.132-.266c-.098-.132-.212-.199-.349-.265a1.93 1.93 0 00-.484-.2h-.016c-.247-.066-.44-.2-.568-.465a1.264 1.264 0 01-.13-.668c.016-.335.148-.666.353-.932.2-.265.432-.466.7-.6h.004a1.38 1.38 0 01.488-.132c.147 0 .302.032.447.132.146.066.287.199.395.4.11.132.18.333.21.533.02.132.027.264.027.4v.002c-.009.2-.045.399-.115.598a2.17 2.17 0 01-.333.6c-.143.198-.322.33-.507.463-.184.132-.374.198-.533.264h-.02c-.066.024-.124.032-.19.066-.06.027-.12.056-.17.088l.008.012c.164.054.32.132.462.2.28.198.506.463.668.732.173.332.256.732.226 1.131-.03.468-.184.935-.446 1.333a2.665 2.665 0 01-.897.866l.01.008c.066.066.098.132.14.2.04.066.076.131.106.198.079.166.12.332.15.532.034.198.046.398.034.598v.009c-.017.198-.046.399-.1.532a.898.898 0 01-.152.266h-.003c-.054.066-.12.132-.198.198a.654.654 0 01-.245.132c-.178.066-.37.098-.57.098-.2 0-.398-.032-.598-.098a1.84 1.84 0 01-.487-.2 1.388 1.388 0 01-.373-.332 1.012 1.012 0 01-.205-.466.902.902 0 01-.009-.132c0-.066.003-.132.012-.2a.69.69 0 01.052-.2c.029-.065.064-.131.106-.198.041-.066.088-.132.14-.198l.024-.03-.112-.1c-.125-.109-.258-.216-.408-.314a3.824 3.824 0 00-.518-.262.798.798 0 00-.2-.058 3.008 3.008 0 01-.2-.066 2.11 2.11 0 01-.348-.13 1.153 1.153 0 01-.265-.199c-.066-.066-.13-.132-.184-.198a.917.917 0 01-.14-.266c-.034-.066-.06-.132-.08-.198a.98.98 0 01-.031-.2.88.88 0 01-.006-.132l.01-.069c.032-.2.096-.398.19-.532.127-.2.288-.399.487-.532.2-.132.414-.2.636-.266.117-.028.235-.042.354-.042l.054-.006c.2-.016.401-.016.6 0 .133.016.264.032.397.066.134.032.266.066.398.132l.065.024c.054-.134.09-.265.106-.4.016-.132.016-.265-.004-.397a1.094 1.094 0 00-.162-.4c-.08-.132-.192-.265-.334-.332-.15-.066-.327-.132-.52-.132-.192 0-.392.066-.6.132-.162.066-.324.132-.476.265-.15.133-.287.333-.384.533a1.538 1.538 0 00-.165.865v.015c0 .2.066.4.151.6.066.132.155.265.262.398.111.133.236.265.366.332.13.066.271.133.397.133l.055.003h.003c.13 0 .263-.035.391-.1.128-.066.253-.132.365-.264.108-.133.2-.333.27-.533.07-.198.11-.463.11-.732v-.006l-.003-.2a4.04 4.04 0 00-.057-.4 1.917 1.917 0 00-.19-.4 1.194 1.194 0 00-.364-.4c-.162-.132-.363-.2-.592-.2l-.024.003h-.004c-.098 0-.196.021-.287.066-.092.033-.178.099-.26.133-.083.066-.161.132-.234.265-.073.133-.14.333-.187.6l-.011.065v.004l-.014.135c-.003.066-.003.133-.003.2v.013c0 .2.028.397.081.598.053.2.137.4.249.533.11.132.248.265.413.332.163.066.352.1.555.1h.069c.2 0 .4-.034.598-.1.2-.066.398-.2.564-.333.165-.132.3-.332.402-.532.1-.2.17-.466.203-.732l.007-.069c.016-.2.019-.465-.015-.665a1.61 1.61 0 00-.166-.533 1.12 1.12 0 00-.354-.4c-.15-.133-.335-.2-.545-.2h-.015c-.2 0-.395.032-.585.132a1.57 1.57 0 00-.486.333c-.143.132-.264.333-.36.532-.096.2-.165.466-.198.732v.003c-.026.2-.023.4.007.598.029.2.08.4.166.533.085.132.186.265.32.398.132.133.29.2.473.266.184.066.392.1.612.1h.02c.213 0 .424-.034.624-.1.2-.066.386-.199.548-.333.162-.133.298-.332.406-.532.109-.2.186-.466.228-.732a2.62 2.62 0 00.034-.4v-.265a2.09 2.09 0 00-.077-.533 1.415 1.415 0 00-.25-.465 1.183 1.183 0 00-.43-.333c-.174-.066-.376-.1-.588-.1h-.016c-.11 0-.224.017-.333.05a1.6 1.6 0 00-.333.133 1.435 1.435 0 00-.32.232c-.1.1-.184.21-.257.333-.073.132-.13.28-.173.4-.037.132-.07.265-.089.397a2.99 2.99 0 00-.035.4v.132c0 .134.016.265.046.398a1.6 1.6 0 00.152.398c.073.133.165.265.278.333.113.066.247.132.397.132h.016c.132 0 .257-.032.374-.1a1.01 1.01 0 00.298-.265c.08-.133.14-.333.177-.532.037-.2.05-.466.034-.732v-.006a2.56 2.56 0 00-.077-.533 1.35 1.35 0 00-.22-.466 1.05 1.05 0 00-.393-.333c-.162-.066-.357-.1-.566-.1h-.02c-.185 0-.365.034-.536.1a1.552 1.552 0 00-.468.266c-.14.132-.262.332-.362.532a2.21 2.21 0 00-.227.732 2.76 2.76 0 00-.038.598v.066c.016.2.05.4.115.6.065.198.16.398.279.532.12.132.267.265.437.332.17.066.367.1.574.1l.037-.002h.003c.183 0 .362-.034.53-.1a1.45 1.45 0 00.45-.265c.13-.133.24-.333.32-.532.082-.2.14-.466.167-.732a3.24 3.24 0 00.02-.4v-.133c-.003-.2-.035-.4-.1-.598a1.47 1.47 0 00-.262-.533 1.15 1.15 0 00-.443-.332c-.18-.066-.385-.1-.604-.1l-.037.003h-.003c-.145 0-.285.021-.42.066a1.2 1.2 0 00-.374.2 1.17 1.17 0 00-.287.333c-.076.133-.133.332-.17.532-.036.2-.053.465-.036.73v.003c.02.2.062.4.127.6.066.198.157.398.27.532.113.132.247.265.4.332.152.066.32.1.495.1h.02c.146 0 .287-.034.42-.1a1.1 1.1 0 00.352-.265c.1-.133.177-.333.231-.532.054-.2.082-.466.079-.732v-.006a2.44 2.44 0 00-.066-.533 1.27 1.27 0 00-.2-.466 1.02 1.02 0 00-.36-.332 1.037 1.037 0 00-.514-.133h-.016l-.037.003z"/>
    </svg>
  );
}

/* ------------------------------------------------------------------ */
/*  Hero                                                               */
/* ------------------------------------------------------------------ */

function Hero() {
  return (
    <section className="pt-32 pb-24 md:pt-44 md:pb-32 px-6 relative overflow-hidden">
      <div className="landing-grid-bg" />

      <div className="max-w-4xl mx-auto text-center relative z-10">
        <div className="landing-hero-badge mb-6 inline-flex items-center gap-2 px-4 py-1.5 rounded-full border border-[var(--color-border)] bg-[var(--color-surface)] text-sm text-[var(--color-text-secondary)]">
          <span className="w-2 h-2 rounded-full bg-[var(--color-green)] animate-pulse" />
          Now in early access
        </div>

        <h1 className="landing-hero-title text-5xl md:text-7xl font-bold text-[var(--color-text)] tracking-tight leading-[1.08] mb-6">
          One IDE. Any agent.
        </h1>

        <p className="landing-hero-subtitle text-lg md:text-xl text-[var(--color-text-secondary)] max-w-2xl mx-auto mb-10 leading-relaxed">
          An agent-agnostic code editor built for extensibility.
          Use Claude Code, Codex, Cursor, Copilot, or your favorite CLI agent.
          Build custom apps with the Tapp framework.
        </p>

        <div className="landing-hero-cta flex flex-col sm:flex-row items-center justify-center gap-4">
          <a
            href="#download"
            className="inline-flex items-center gap-2 bg-[var(--color-text)] text-white font-medium px-8 py-3.5 rounded-xl text-base hover:opacity-90 transition-all active:scale-[0.98]"
          >
            <IconDownload />
            Download for Free
          </a>
          <Link
            to="/docs"
            className="inline-flex items-center gap-2 text-[var(--color-text-secondary)] font-medium px-8 py-3.5 rounded-xl text-base border border-[var(--color-border)] hover:bg-[var(--color-surface)] transition-colors"
          >
            View Documentation
            <IconArrowRight />
          </Link>
        </div>
      </div>

      {/* IDE Preview */}
      <div className="landing-hero-visual max-w-6xl mx-auto mt-16 px-6">
        <div className="rounded-2xl border border-[#2a2a2a] bg-[#1a1a1a] shadow-2xl overflow-hidden">
          {/* Window chrome */}
          <div className="flex items-center gap-2 px-4 py-3 border-b border-white/10">
            <div className="flex gap-1.5">
              <div className="w-3 h-3 rounded-full bg-[#ff5f57]" />
              <div className="w-3 h-3 rounded-full bg-[#febc2e]" />
              <div className="w-3 h-3 rounded-full bg-[#28c840]" />
            </div>
            <div className="flex items-center gap-2 ml-4">
              <div className="px-3 py-1 rounded-md bg-white/10 text-xs text-white/80 font-medium">Review</div>
              <div className="px-3 py-1 rounded-md text-xs text-white/40">Editor</div>
            </div>
            <div className="flex-1" />
            <div className="flex items-center gap-2 text-xs text-white/40">
              <span className="flex items-center gap-1 px-2 py-1 rounded bg-[#28c840]/20 text-[#28c840]">
                <span className="w-1.5 h-1.5 rounded-full bg-[#28c840]" />
                AI
              </span>
              <span>CTX</span>
            </div>
          </div>
          
          {/* Main content - 3 panel layout */}
          <div className="flex h-[400px]">
            {/* Left panel - Review Changes */}
            <div className="w-56 flex-shrink-0 border-r border-white/10 p-4 hero-panel-left">
              <div className="flex items-center justify-between mb-3">
                <span className="text-xs font-medium text-white/80">Review Changes</span>
                <span className="px-1.5 py-0.5 rounded bg-[#7BA8C9]/20 text-[#7BA8C9] text-[10px] whitespace-nowrap">2 pending</span>
                <span className="text-white/30 ml-2">×</span>
              </div>
              <div className="flex gap-4 mb-4 text-xs">
                <span className="text-white/50">Accept All</span>
                <span className="text-white/50">Reject All</span>
              </div>
              {/* File list */}
              <div className="space-y-1 mt-4">
                <div className="flex items-center justify-between text-xs py-2 px-2 rounded bg-white/5">
                  <div className="flex items-center gap-2 min-w-0">
                    <span className="text-[#C9B87B] flex-shrink-0">M</span>
                    <span className="text-white/70 truncate">src/App.jsx</span>
                  </div>
                  <div className="flex items-center gap-2 flex-shrink-0 ml-2">
                    <span className="text-[#7BC9A0]">✓</span>
                    <span className="text-white/30">×</span>
                  </div>
                </div>
                <div className="text-[10px] text-[#7BC9A0] pl-6">+2</div>
                <div className="flex items-center justify-between text-xs py-2 px-2 rounded hover:bg-white/5">
                  <div className="flex items-center gap-2 min-w-0">
                    <span className="text-[#7BC9A0] flex-shrink-0">A</span>
                    <span className="text-white/50 truncate">Newsletter.css</span>
                  </div>
                  <div className="flex items-center gap-2 flex-shrink-0 ml-2">
                    <span className="text-white/30">✓</span>
                    <span className="text-white/30">×</span>
                  </div>
                </div>
              </div>
            </div>
            
            {/* Center - Editor with diff */}
            <div className="flex-1 flex flex-col hero-panel-center overflow-hidden">
              {/* File tab bar */}
              <div className="flex items-center justify-between px-4 py-2 border-b border-white/10">
                <span className="text-xs text-white/70">src/App.jsx</span>
                <div className="flex items-center gap-2 text-xs">
                  <span className="text-white/40">Inline</span>
                  <span className="px-2 py-0.5 rounded bg-[#7BC9A0]/20 text-[#7BC9A0]">Accept</span>
                  <span className="px-2 py-0.5 rounded bg-[#C97B7B]/20 text-[#C97B7B]">Reject</span>
                </div>
              </div>
              {/* Code with diff */}
              <div className="flex-1 p-4 font-mono text-xs overflow-hidden">
                <div className="flex">
                  {/* Line numbers */}
                  <div className="text-white/20 text-right pr-4 select-none space-y-0.5">
                    <div>1</div><div>2</div><div>3</div><div>4</div><div>5</div><div>6</div><div>7</div><div>8</div><div>9</div><div>10</div><div>11</div><div>12</div><div>13</div>
                  </div>
                  {/* Code content */}
                  <div className="flex-1 space-y-0.5 overflow-hidden">
                    <div className="text-white/60"><span className="text-[#A07BC9]">import</span> <span className="text-[#C97B7B]">'./App.css'</span></div>
                    <div className="text-white/60"><span className="text-[#A07BC9]">import</span> Navbar <span className="text-[#A07BC9]">from</span> <span className="text-[#C97B7B]">'./components/Navbar'</span></div>
                    <div className="text-white/60"><span className="text-[#A07BC9]">import</span> Hero <span className="text-[#A07BC9]">from</span> <span className="text-[#C97B7B]">'./components/Hero'</span></div>
                    <div className="text-white/60"><span className="text-[#A07BC9]">import</span> Features <span className="text-[#A07BC9]">from</span> <span className="text-[#C97B7B]">'./components/Features'</span></div>
                    <div className="text-white/60"><span className="text-[#A07BC9]">import</span> Stats <span className="text-[#A07BC9]">from</span> <span className="text-[#C97B7B]">'./components/Stats'</span></div>
                    <div className="text-white/60"><span className="text-[#A07BC9]">import</span> Testimonials <span className="text-[#A07BC9]">from</span> <span className="text-[#C97B7B]">'./components/Testimonials'</span></div>
                    <div className="bg-[#7BC9A0]/10 text-[#7BC9A0] px-2 -mx-2 border-l-2 border-[#7BC9A0] hero-diff-add"><span className="text-[#A07BC9]">import</span> Newsletter <span className="text-[#A07BC9]">from</span> <span className="text-[#C97B7B]">'./components/Newsletter'</span></div>
                    <div className="text-white/60"><span className="text-[#A07BC9]">import</span> CTA <span className="text-[#A07BC9]">from</span> <span className="text-[#C97B7B]">'./components/CTA'</span></div>
                    <div className="text-white/60"><span className="text-[#A07BC9]">import</span> Footer <span className="text-[#A07BC9]">from</span> <span className="text-[#C97B7B]">'./components/Footer'</span></div>
                    <div className="text-white/30">&nbsp;</div>
                    <div className="text-white/60"><span className="text-[#A07BC9]">function</span> <span className="text-[#7BA8C9]">App</span>() {'{'}</div>
                    <div className="text-white/60">  <span className="text-[#A07BC9]">return</span> (</div>
                    <div className="text-white/60">    &lt;<span className="text-[#7BC9A0]">div</span> <span className="text-[#7BA8C9]">className</span>=<span className="text-[#C97B7B]">"App"</span>&gt;</div>
                  </div>
                </div>
              </div>
            </div>
            
            {/* Right panel - Agent */}
            <div className="w-64 border-l border-white/10 p-4 hero-panel-right">
              <div className="flex items-center gap-3 mb-4 p-3 rounded-lg bg-white/5 hero-agent-card">
                {/* Claude Code icon */}
                <div className="w-10 h-10 rounded-lg bg-[#da7756] flex items-center justify-center">
                  <svg width="20" height="16" viewBox="0 0 20 16" fill="none">
                    <rect x="2" y="2" width="4" height="4" fill="#1a1a1a" />
                    <rect x="8" y="2" width="4" height="4" fill="#1a1a1a" />
                    <rect x="14" y="2" width="4" height="4" fill="#1a1a1a" />
                    <rect x="2" y="10" width="4" height="4" fill="#1a1a1a" />
                    <rect x="8" y="10" width="4" height="4" fill="#1a1a1a" />
                    <rect x="14" y="10" width="4" height="4" fill="#1a1a1a" />
                  </svg>
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-xs text-white/80 font-medium">Claude Code <span className="text-white/40">v2.1.76</span></div>
                  <div className="text-[10px] text-white/40 truncate">Opus 4.6 (1M context...</div>
                  <div className="text-[10px] text-white/40">Claude Team</div>
                </div>
              </div>
              <div className="flex items-center gap-2 text-white/50 hero-cursor">
                <span className="text-sm">›</span>
                <span className="w-2 h-4 bg-white/50 animate-pulse" />
              </div>
            </div>
          </div>
          
          {/* Bottom bar */}
          <div className="flex items-center justify-between px-4 py-2 border-t border-white/10 text-xs">
            <div className="flex items-center gap-3">
              <span className="flex items-center gap-1 px-2 py-1 rounded bg-white/5 text-white/50">
                <span className="text-[#7BA8C9]">◆</span> Context
              </span>
              <span className="px-2 py-1 rounded bg-[#7BA8C9]/20 text-[#7BA8C9]">AI ↗</span>
              <span className="text-white/30">$ Run a shell command...</span>
            </div>
            <div className="flex items-center gap-4">
              <span className="text-white/50">⌥ feature/test</span>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded-full bg-[#7BA8C9]" />
                <span className="text-white/50">23 changes</span>
              </span>
              <span className="text-[#7BA8C9]">○ Sync</span>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------------ */
/*  Features                                                           */
/* ------------------------------------------------------------------ */

const features = [
  {
    icon: <IconBrain />,
    label: 'Agent Agnostic',
    title: 'Bring Your Own Agent',
    description:
      'Works with any AI coding agent — Claude Code, Codex, Cursor, Copilot, Aider, or your custom CLI tools. No vendor lock-in, full flexibility.',
    color: '#A07BC9',
  },
  {
    icon: <IconPuzzle />,
    label: 'Extensible',
    title: 'Tapp Extension Framework',
    description:
      'Build powerful apps with the Tapp SDK. WASM-based, secure, and fast. Create custom tools, panels, and workflows that integrate seamlessly.',
    color: '#7BC9A0',
  },
  {
    icon: <IconPalette />,
    label: 'Customizable',
    title: 'Themes & Personalization',
    description:
      'Make it yours with custom themes. Define colors, fonts, and UI density. Share themes with the community or keep them private.',
    color: '#7BA8C9',
  },
  {
    icon: <IconSpeed />,
    label: 'Performance',
    title: 'Fast & Lightweight',
    description:
      'Built with Tauri and Rust for native performance. Lightning-fast startup, minimal memory footprint, and smooth editing even in large projects.',
    color: '#C9B87B',
  },
  {
    icon: <IconCode />,
    label: 'Developer Experience',
    title: 'Modern Editing Features',
    description:
      'Intelligent code navigation, multi-cursor editing, integrated terminal, Git support, and all the features you expect from a modern editor.',
    color: '#C97B7B',
  },
  {
    icon: <IconShield />,
    label: 'Security',
    title: 'Sandboxed Extensions',
    description:
      'Tapp extensions run in isolated WASM sandboxes with explicit permissions. Your code and data stay safe, even with third-party apps.',
    color: '#7BC9C9',
  },
];

function Features() {
  return (
    <section id="features" className="py-24 px-6 bg-[var(--color-surface)]">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-16">
          <p className="text-sm font-medium text-[var(--color-accent)] tracking-wide uppercase mb-3">Features</p>
          <h2 className="text-3xl md:text-4xl font-bold text-[var(--color-text)] tracking-tight">
            Everything you need to
            <br />build software faster
          </h2>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature) => (
            <div
              key={feature.label}
              className="landing-feature-card group relative rounded-2xl border border-[var(--color-border)] p-8 hover:border-[var(--color-border-focus)] transition-all duration-300 bg-[var(--color-surface)]"
            >
              <div
                className="w-12 h-12 rounded-xl flex items-center justify-center mb-5 transition-colors"
                style={{ backgroundColor: `color-mix(in srgb, ${feature.color} 12%, transparent)`, color: feature.color }}
              >
                {feature.icon}
              </div>
              <p className="text-xs font-medium tracking-wide uppercase text-[var(--color-text-muted)] mb-2">{feature.label}</p>
              <h3 className="text-xl font-semibold text-[var(--color-text)] mb-3">{feature.title}</h3>
              <p className="text-[var(--color-text-secondary)] leading-relaxed">{feature.description}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------------ */
/*  Download Section                                                   */
/* ------------------------------------------------------------------ */

const platforms = [
  {
    name: 'macOS',
    icon: <IconApple />,
    description: 'Apple Silicon & Intel',
    primary: 'Download for macOS',
    links: [
      { label: 'Apple Silicon (.dmg)', href: '#' },
      { label: 'Intel (.dmg)', href: '#' },
    ],
    color: '#1C1917',
  },
  {
    name: 'Windows',
    icon: <IconWindows />,
    description: 'Windows 10/11',
    primary: 'Download for Windows',
    links: [
      { label: 'x64 Installer (.msi)', href: '#' },
      { label: 'x64 Portable (.zip)', href: '#' },
    ],
    color: '#0078D4',
  },
  {
    name: 'Linux',
    icon: <IconLinux />,
    description: 'Ubuntu, Fedora, Arch',
    primary: 'Download for Linux',
    links: [
      { label: '.deb (Ubuntu/Debian)', href: '#' },
      { label: '.rpm (Fedora/RHEL)', href: '#' },
      { label: '.AppImage', href: '#' },
    ],
    color: '#FCC624',
  },
];

function Download() {
  return (
    <section id="download" className="py-24 px-6">
      <div className="max-w-5xl mx-auto">
        <div className="text-center mb-16">
          <p className="text-sm font-medium text-[var(--color-accent)] tracking-wide uppercase mb-3">Download</p>
          <h2 className="text-3xl md:text-4xl font-bold text-[var(--color-text)] tracking-tight">
            Get Tyck for your platform
          </h2>
          <p className="text-lg text-[var(--color-text-secondary)] mt-4 max-w-xl mx-auto">
            Free and open source. Available for macOS, Windows, and Linux.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {platforms.map((platform) => (
            <div
              key={platform.name}
              className="platform-card rounded-2xl border border-[var(--color-border)] p-8 text-center"
            >
              <div
                className="w-16 h-16 rounded-2xl mx-auto mb-6 flex items-center justify-center"
                style={{ backgroundColor: `color-mix(in srgb, ${platform.color} 10%, transparent)`, color: platform.color }}
              >
                {platform.icon}
              </div>
              <h3 className="text-xl font-semibold text-[var(--color-text)] mb-1">{platform.name}</h3>
              <p className="text-sm text-[var(--color-text-muted)] mb-6">{platform.description}</p>
              
              <a
                href={platform.links[0].href}
                className="inline-flex items-center justify-center gap-2 w-full bg-[var(--color-text)] text-white font-medium px-6 py-3 rounded-xl text-sm hover:opacity-90 transition-all active:scale-[0.98] mb-4"
              >
                <IconDownload />
                {platform.primary}
              </a>

              <div className="space-y-2">
                {platform.links.map((link) => (
                  <a
                    key={link.label}
                    href={link.href}
                    className="block text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-text)] transition-colors"
                  >
                    {link.label}
                  </a>
                ))}
              </div>
            </div>
          ))}
        </div>

        <p className="text-center text-sm text-[var(--color-text-muted)] mt-8">
          Or build from source: <a href="https://github.com/tyck-ai/ide" target="_blank" rel="noopener noreferrer" className="text-[var(--color-blue)] hover:underline">View on GitHub</a>
        </p>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------------ */
/*  CTA                                                                */
/* ------------------------------------------------------------------ */

function CTA() {
  return (
    <section className="landing-cta-section relative overflow-hidden bg-[var(--color-text)]">
      <div className="absolute inset-0 opacity-[0.03]" style={{
        backgroundImage: 'radial-gradient(circle at 1px 1px, white 1px, transparent 0)',
        backgroundSize: '24px 24px',
      }} />

      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[600px] h-px cta-glow-line" />

      <div className="relative z-10 max-w-4xl mx-auto px-6 py-24 md:py-32 text-center">
        <h2 className="text-4xl md:text-5xl font-bold text-white tracking-tight mb-4 leading-[1.1]">
          Ready to build
          <br />something amazing?
        </h2>
        <p className="text-lg text-white/50 max-w-xl mx-auto mb-10 leading-relaxed">
          Download Tyck and start coding with your favorite AI agent, custom extensions, and a beautiful interface.
        </p>

        <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
          <a
            href="#download"
            className="inline-flex items-center gap-2 bg-white text-[var(--color-text)] font-medium px-8 py-3.5 rounded-xl text-base hover:bg-white/90 transition-colors active:scale-[0.98]"
          >
            <IconDownload />
            Download Now
          </a>
          <Link
            to="/docs"
            className="inline-flex items-center gap-2 text-white/70 font-medium px-8 py-3.5 rounded-xl text-base border border-white/20 hover:bg-white/10 transition-colors"
          >
            Read the Docs
            <IconArrowRight />
          </Link>
        </div>
      </div>
    </section>
  );
}

/* ------------------------------------------------------------------ */
/*  Page                                                               */
/* ------------------------------------------------------------------ */

export function HomePage() {
  return (
    <div className="min-h-screen bg-[var(--color-background)]">
      <Navbar />
      <Hero />
      <Features />
      <Download />
      <CTA />
      <Footer />
    </div>
  );
}
