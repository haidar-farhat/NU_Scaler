<?php

namespace App\Notifications;

use Illuminate\Bus\Queueable;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Notifications\Messages\MailMessage;
use Illuminate\Notifications\Notification;

class NewFeedbackNotification extends Notification implements ShouldQueue
{
    use Queueable;

    /**
     * The feedback instance.
     *
     * @var mixed
     */
    protected $feedback;

    /**
     * The feedback type.
     *
     * @var string
     */
    protected $type;

    /**
     * Create a new notification instance.
     *
     * @param mixed $feedback
     * @param string $type
     * @return void
     */
    public function __construct($feedback, string $type)
    {
        $this->feedback = $feedback;
        $this->type = $type;
    }

    /**
     * Get the notification's delivery channels.
     *
     * @return array<int, string>
     */
    public function via(object $notifiable): array
    {
        return ['mail', 'database'];
    }

    /**
     * Get the mail representation of the notification.
     */
    public function toMail(object $notifiable): MailMessage
    {
        $subject = match($this->type) {
            'review' => 'New User Review Submitted',
            'bug-report' => 'New Bug Report Submitted',
            'hardware-survey' => 'New Hardware Survey Submitted',
            default => 'New Feedback Submitted',
        };

        return (new MailMessage)
                    ->subject($subject)
                    ->greeting('Hello Admin,')
                    ->line("A new {$this->type} has been submitted.")
                    ->action('View Details', url("/admin/dashboard/{$this->type}s/{$this->feedback->id}"))
                    ->line('Thank you for using Nu Scaler!');
    }

    /**
     * Get the array representation of the notification.
     *
     * @return array<string, mixed>
     */
    public function toArray(object $notifiable): array
    {
        return [
            'feedback_id' => $this->feedback->id,
            'type' => $this->type,
            'message' => "New {$this->type} submitted by user #{$this->feedback->user_uuid}",
            'created_at' => $this->feedback->created_at,
        ];
    }
}
