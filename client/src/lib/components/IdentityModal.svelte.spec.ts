// @vitest-environment jsdom
import { beforeEach, it, expect, describe, vi, afterEach } from 'vitest';
import { render, fireEvent, screen, cleanup } from '@testing-library/svelte';
import Modal from './IdentityModal.svelte';
import * as matchers from '@testing-library/jest-dom/matchers';

expect.extend(matchers);

describe('Modal Component', () => {
  const mockProps = {
    show: true,
    isLoading: false,
    onConfirm: vi.fn(),
    onCancel: vi.fn(),
    username: '',
    password: '',
    handleSubmit: vi.fn(),
    form: null
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  })

  it('renders when show is true', () => {
    render(Modal, { props: mockProps });
    const modalText = screen.getByText(/A secure key pair will be generated for you/);
    expect(modalText).toBeInTheDocument();
  });

  it('does not render when show is false', () => {
    render(Modal, { props: { ...mockProps, show: false } });
    expect(screen.queryByText('Create New Identity')).not.toBeInTheDocument();
  });

  it('displays the correct title and description', () => {
    render(Modal, { props: mockProps });
    expect(screen.getByText('Create New Identity')).toBeInTheDocument();
    expect(screen.getByText(/A secure key pair will be generated for you/)).toBeInTheDocument();
  });

  it('calls onCancel when clicking outside the modal', async () => {
    render(Modal, { props: mockProps });
    const backdrop = screen.getByRole('button', { name: 'Cancel' });
    await fireEvent.click(backdrop);
    expect(mockProps.onCancel).toHaveBeenCalled();
  });

  it('calls onCancel when pressing Escape key', async () => {
    render(Modal, { props: mockProps });
    const backdrop = screen.getByRole('button', { name: 'Cancel' });
    await fireEvent.keyDown(backdrop, { key: 'Escape' });
    expect(mockProps.onCancel).toHaveBeenCalled();
  });

  it('does not call onCancel when clicking inside the modal', async () => {
    render(Modal, { props: mockProps });
    const modalContent = screen.getByText('Create New Identity', {  });
    await fireEvent.click(modalContent);
    expect(mockProps.onCancel).not.toHaveBeenCalled();
  });

  it('calls onCancel when clicking the cancel button', async () => {
    render(Modal, { props: mockProps });
    const cancelButton = screen.getByText('Cancel');
    await fireEvent.click(cancelButton);
    expect(mockProps.onCancel).toHaveBeenCalled();
  });

  it('submits the form with username and password', async () => {
    render(Modal, { props: mockProps });
    
    const usernameInput = screen.getByPlaceholderText('Optional Username');
    const passwordInput = screen.getByPlaceholderText('Password');
    const submitButton = screen.getByText('Confirm');
    
    await fireEvent.input(usernameInput, { target: { value: 'testuser' } });
    await fireEvent.input(passwordInput, { target: { value: 'securepassword' } });
    await fireEvent.click(submitButton);
    
    expect(mockProps.handleSubmit).toHaveBeenCalled();
  });

  it('shows loading state when isLoading is true', async () => {
    render(Modal, { props: { ...mockProps, isLoading: true } });
    expect(screen.getByText('Creating...')).toBeInTheDocument();
    expect(screen.getByTestId('spinner')).toBeInTheDocument();
  });

  it('disables buttons when isLoading is true', async () => {
    render(Modal, { props: { ...mockProps, isLoading: true } });
    expect(screen.getByText('Cancel')).toBeDisabled();
    expect(screen.getByText('Creating...').closest('button')).toBeDisabled();
  });

  it('shows form errors when present', async () => {
    const errorProps = {
      ...mockProps,
      form: {
        error: true,
        errors: [
          { message: 'Password is too weak' },
          { message: 'Username already exists' }
        ]
      }
    };
    
    render(Modal, { props: errorProps });
    
    expect(screen.getByText('Password is too weak')).toBeInTheDocument();
    expect(screen.getByText('Username already exists')).toBeInTheDocument();
  });

  it('requires password field', async () => {
    render(Modal, { props: mockProps });
    const passwordInput = screen.getByPlaceholderText('Password');
    expect(passwordInput).toBeRequired();
  });

  it('binds username and password values', async () => {
    let username = $state('');
    let password = $state('');

    const bindableProps = {
      ...mockProps,
      get username() {
        return username;
      },
      set username(value: string) {
        username = value;
      },
      get password() {
        return password;
      },
      set password(value: string) {
        password = value;
      }
    };

    render(Modal, { props: bindableProps });

    const usernameInput = screen.getByPlaceholderText('Optional Username');
    const passwordInput = screen.getByPlaceholderText('Password');

    await fireEvent.input(usernameInput, { target: { value: 'newuser' } });
    await fireEvent.input(passwordInput, { target: { value: 'newpass' } });

    expect(username).toBe('newuser');
    expect(password).toBe('newpass');
  });
});
