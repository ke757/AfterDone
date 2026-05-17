import { Button } from '@/components/ui/button';
import { FolderOpen, RefreshCw } from 'lucide-react';
import { useUIStore } from '@/stores/uiStore';

export function QuickStartPage() {
  const currentRepoPath = useUIStore((s) => s.currentRepoPath);
  const currentRepoName = useUIStore((s) => s.currentRepoName);
  const needRestart = useUIStore((s) => s.needRestart);
  const initError = useUIStore((s) => s.initError);
  const selectRepo = useUIStore((s) => s.selectRepo);
  const completeInit = useUIStore((s) => s.completeInit);

  if (needRestart) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="flex max-w-md flex-col items-center gap-6 rounded-lg border border-border bg-card p-8 text-center shadow-sm">
          <div className="flex size-16 items-center justify-center rounded-full bg-primary/10">
            <span className="text-2xl font-bold text-primary">AD</span>
          </div>
          <h2 className="text-xl font-semibold text-foreground">AfterDone</h2>
          <div className="flex flex-col items-center gap-3">
            <div className="rounded-md border border-primary/30 bg-primary/5 px-4 py-3 text-sm text-primary">
              ResourceRepo 已设置为「{currentRepoName}」，请重启应用以生效。
            </div>
            <Button variant="outline" size="lg" className="gap-2" onClick={() => window.close()}>
              <RefreshCw className="size-4" />
              关闭应用
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full items-center justify-center">
      <div className="flex max-w-md flex-col items-center gap-6 rounded-lg border border-border bg-card p-8 text-center shadow-sm">
        <div className="flex size-16 items-center justify-center rounded-full bg-primary/10">
          <span className="text-2xl font-bold text-primary">AD</span>
        </div>
        <div>
          <h2 className="text-xl font-semibold text-foreground">欢迎使用 AfterDone</h2>
          <p className="mt-2 text-sm text-muted-foreground">
            所有工作空间的元数据数据库和资源将存储在以下路径中。
          </p>
        </div>

        <div className="w-full rounded-md border border-border bg-muted/50 px-4 py-3">
          <p className="text-xs text-muted-foreground mb-1">ResourceRepo 路径</p>
          <p className="text-sm font-mono text-foreground break-all">{currentRepoPath}</p>
        </div>

        {initError && (
          <p className="text-sm text-destructive">{initError}</p>
        )}

        <div className="flex flex-col gap-2 w-full">
          <Button variant="outline" className="gap-2" onClick={selectRepo}>
            <FolderOpen className="size-4" />
            自定义路径
          </Button>
          <Button size="lg" onClick={completeInit}>
            开始使用
          </Button>
        </div>
      </div>
    </div>
  );
}