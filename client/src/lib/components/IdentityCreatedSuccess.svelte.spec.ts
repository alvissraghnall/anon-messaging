import { render, fireEvent } from '@testing-library/svelte';
import { describe, it, expect, vi } from 'vitest';
import IdentityDialog from './IdentityCreatedSuccess.svelte';

describe('IdentityDialog', () => {
  const mockProps = {
    id: 'abc123',
    username: 'testuser',
    onClose: vi.fn(),
    onStartMessaging: vi.fn(),
  };

  it('renders username and ID', () => {
    const { getByText } = render(IdentityDialog, { props: mockProps });

    expect(getByText(/testuser/)).toBeTruthy();
  });

  it('calls onStartMessaging when the button is clicked', async () => {
    const { getByText } = render(IdentityDialog, { props: mockProps });

    const startButton = getByText(/start receiving and sending messages/i);
    await fireEvent.click(startButton);

    expect(mockProps.onStartMessaging).toHaveBeenCalled();
  });

  it('calls onClose when overlay is clicked', async () => {
    const { container } = render(IdentityDialog, { props: mockProps });

    const overlay = container.querySelector('[tabindex="-3"]');
    await fireEvent.click(overlay!);

    expect(mockProps.onClose).toHaveBeenCalled();
  });

  it('does NOT call onClose when inner dialog is clicked', async () => {
    const { container, getByTestId } = render(IdentityDialog, { props: mockProps });

    const dialog = getByTestId(/inner/)!;
    await fireEvent.click(dialog);

    expect(mockProps.onClose).not.toHaveBeenCalled();
  });

  it('calls onClose when Close button is clicked', async () => {
    const { getByText } = render(IdentityDialog, { props: mockProps });

    const closeButton = getByText(/Close/i);
    await fireEvent.click(closeButton);

    expect(mockProps.onClose).toHaveBeenCalled();
  });
});
