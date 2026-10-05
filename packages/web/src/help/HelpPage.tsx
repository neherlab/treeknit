export function HelpPage() {
  return (
    <main className="min-h-0 flex-1 overflow-y-auto">
      <article className="flex max-w-[75ch] flex-col gap-4 px-6 py-8 text-base">
        <h1 className="text-lg font-semibold">Help</h1>
        <section aria-labelledby="help-privacy" className="flex flex-col gap-2">
          <h2 id="help-privacy" className="text-base font-semibold">
            Privacy
          </h2>
          <p>
            Your trees stay in this browser. TreeKnit sends nothing to a server. They are saved between visits only when
            you turn on Keep my workspace in this browser.
          </p>
        </section>
      </article>
    </main>
  );
}
