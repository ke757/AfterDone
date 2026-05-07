import { useEffect } from "react";
import { useGoalStore, useAgentStore, initAllListeners, cleanupAllListeners } from "./stores";
import { Toaster } from "sonner";

function App() {
  const fetchGoals = useGoalStore((state) => state.fetchGoals);
  const fetchStatus = useAgentStore((state) => state.fetchStatus);

  useEffect(() => {
    // 初始化事件监听器
    initAllListeners();

    // 获取初始数据
    fetchGoals();
    fetchStatus();

    return () => {
      cleanupAllListeners();
    };
  }, [fetchGoals, fetchStatus]);

  return (
    <div className="min-h-screen bg-background text-foreground">
      <header className="border-b border-border px-6 py-4">
        <h1 className="text-2xl font-bold text-primary">AfterDone</h1>
        <p className="text-sm text-muted-foreground">Goal Persistence Agent Orchestrator</p>
      </header>

      <main className="container mx-auto p-6">
        <div className="grid gap-6">
          {/* Goal List Section */}
          <section className="rounded-lg border border-border bg-card p-6">
            <h2 className="text-lg font-semibold mb-4">Goals</h2>
            <p className="text-muted-foreground">
              Create and manage your goals. Each goal represents a persistent objective
              that can be explored through the agent workbench.
            </p>
            <div className="mt-4">
              <button className="px-4 py-2 bg-primary text-primary-foreground rounded-md hover:bg-primary/90 transition-colors">
                New Goal
              </button>
            </div>
          </section>

          {/* Status Section */}
          <section className="rounded-lg border border-border bg-card p-6">
            <h2 className="text-lg font-semibold mb-4">Agent Status</h2>
            <div className="flex items-center gap-2">
              <div className="w-2 h-2 rounded-full bg-yellow-500"></div>
              <span className="text-sm text-muted-foreground">System Ready</span>
            </div>
          </section>
        </div>
      </main>

      <Toaster position="bottom-right" />
    </div>
  );
}

export default App;
