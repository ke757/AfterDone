import { Button } from '@/components/ui/button';
import { Plus } from 'lucide-react';
import { workspaceInit } from '@/services/commands';

export function NewWorkspaceButton() {
  const handleCreate = async () => {
    try {
      await workspaceInit();
    } catch (e) {
      console.error('Failed to create workspace:', e);
    }
  };

  return (
    <Button variant="outline" size="sm" className="w-full" onClick={handleCreate}>
      <Plus className="mr-1 size-3.5" />
      新建工作空间
    </Button>
  );
}