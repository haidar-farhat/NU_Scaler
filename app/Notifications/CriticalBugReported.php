<?php

namespace App\Notifications;

use App\Models\BugReport;
use Illuminate\Bus\Queueable;
use Illuminate\Contracts\Queue\ShouldQueue;
use Illuminate\Notifications\Messages\MailMessage;
use Illuminate\Notifications\Notification;

class CriticalBugReported extends Notification implements ShouldQueue
{
    use Queueable;

    /**
     * The bug report instance.
     *
     * @var \App\Models\BugReport
     */
    private $bugReport;

    /**
     * Create a new notification instance.
     */
    public function __construct(BugReport $bugReport)
    {
        $this->bugReport = $bugReport;
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
        return (new MailMessage)
            ->subject('Critical Bug Report Submitted')
            ->greeting('Critical Bug Alert!')
            ->line('A critical bug report has been submitted for Nu Scaler.')
            ->line('Severity: ' . $this->bugReport->severity)
            ->line('Description: ' . $this->bugReport->description)
            ->line('Steps to reproduce: ' . $this->bugReport->steps_to_reproduce)
            ->action('View Bug Report', url('/admin/bug-reports/' . $this->bugReport->id))
            ->line('Please address this issue as soon as possible.');
    }

    /**
     * Get the array representation of the notification.
     *
     * @return array<string, mixed>
     */
    public function toArray(object $notifiable): array
    {
        return [
            'bug_report_id' => $this->bugReport->id,
            'severity' => $this->bugReport->severity,
            'description' => $this->bugReport->description,
            'timestamp' => now()->toIso8601String(),
        ];
    }
} 