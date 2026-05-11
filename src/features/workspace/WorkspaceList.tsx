import { useEffect, useState } from 'react';
import { useWorkSpaceStore } from '@/stores';
import { workspaceList } from '@/services/commands';

interface WorkSpaceItem {
  id: string;
  goal_id: string;
  current_node_id: string | null;
  goal_md: string | null;
  plan_md: string | null;
  created_at: string;
  updated_at: string;
}

export function WorkspaceList() {
  const currentWorkspace = useWorkSpaceStore((s) => s.current);
  const [workspaces, setWorkspaces] = useState<WorkSpaceItem[]>([]);

  useEffect(() => {
    workspaceList().then(setWorkspaces).catch(() => {});
  }, []);

  const displayList = currentWorkspace
    ? [currentWorkspace, ...workspaces.filter((w) => w.id !== currentWorkspace.id)]
    : workspaces;

  if (displayList.length === 0) {
    return (
      <div className="flex h-full items-center justify-center p-4 text-xs text-muted-foreground">
        暂无工作空间
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-1 p-2">
      {displayList.map((ws) => (
        <div
          key={ws.id}
          className="cursor-pointer rounded-md px-2 py-1.5 text-sm hover:bg-accent"
        >
          <span className="truncate">{ws.goal_md ?? `工作空间 ${ws.id.slice(0, 8)}`}</span>
        </div>
      ))}
    </div>
  );
}