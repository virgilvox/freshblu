import type { Meta, StoryObj } from '@storybook/svelte';
import Tabs from './Tabs.svelte';

const meta = {
  title: 'UI/Tabs',
  component: Tabs,
  tags: ['autodocs'],
} satisfies Meta<Tabs>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  args: {
    tabs: ['Properties', 'Credentials', 'Permissions', 'Webhooks'],
    active: 'Properties',
    children: 'Tab content goes here.' as any,
  },
};
