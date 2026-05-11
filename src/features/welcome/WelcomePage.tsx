import { Button } from '@/components/ui/button';
import { FolderOpen, RefreshCw } from 'lucide-react';
import { useUIStore } from '@/stores/uiStore';

export function WelcomePage() {
  const selectRepo = useUIStore((s) => s.selectRepo);
  const needRestart = useUIStore((s) => s.needRestart);
  const currentRepoName = useUIStore((s) => s.currentRepoName);

  return (
    <div className="flex h-full items-center justify-center">
      <div className="flex max-w-md flex-col items-center gap-6 rounded-lg border border-border bg-card p-8 text-center shadow-sm">
        <div className="flex size-16 items-center justify-center rounded-full bg-primary/10">
          <FolderOpen className="size-8 text-primary" />
        </div>
        <div>
          <h2 className="text-xl font-semibold text-foreground">
            欢迎使用 AfterDone
          </h2>
          <p className="mt-2 text-sm text-muted-foreground">
            请选择一个文件夹作为数据仓库，所有工作空间和目标数据将存储在该目录中。
          </p>
        </div>

        {needRestart ? (
          <div className="flex flex-col items-center gap-3">
            <div className="rounded-md border border-primary/30 bg-primary/5 px-4 py-3 text-sm text-primary">
              数据仓库已设置为「{currentRepoName}」，请重启应用以生效。
            </div>
            <Button
              variant="outline"
              size="lg"
              className="gap-2"
              onClick={() => window.close()}
            >
              <RefreshCw className="size-4" />
              关闭应用
            </Button>
          </div>
        ) : (
          <Button size="lg" onClick={selectRepo}>
            <FolderOpen className="mr-2 size-4" />
            选择文件夹
          </Button>
        )}
      </div>
    </div>
  );
}