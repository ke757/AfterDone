export function WelcomePage() {
  return (
    <div className="flex h-full items-center justify-center">
      <div className="flex max-w-md flex-col items-center gap-6 rounded-lg border border-border bg-card p-8 text-center shadow-sm">
        <div className="flex size-16 items-center justify-center rounded-full bg-primary/10">
          <span className="text-2xl font-bold text-primary">AD</span>
        </div>
        <h2 className="text-xl font-semibold text-foreground">AfterDone</h2>
        <p className="text-sm text-muted-foreground">
          Now you can open a workspace.
        </p>
      </div>
    </div>
  );
}