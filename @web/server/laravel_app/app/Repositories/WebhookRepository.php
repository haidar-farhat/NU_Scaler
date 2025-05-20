<?php

namespace App\Repositories;

use App\Models\Webhook;

class WebhookRepository
{
    public function create(array $data): Webhook
    {
        return Webhook::create($data);
    }

    public function update(Webhook $webhook, array $data): Webhook
    {
        $webhook->fill($data);
        $webhook->save();
        return $webhook;
    }

    public function delete(Webhook $webhook): void
    {
        $webhook->delete();
    }

    public function findByUser($userId)
    {
        return Webhook::where('user_id', $userId)->latest()->paginate();
    }
}
