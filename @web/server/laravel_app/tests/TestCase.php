<?php

namespace Tests;

use Illuminate\Foundation\Testing\TestCase as BaseTestCase;
use Illuminate\Foundation\Testing\LazilyRefreshDatabase;
use Illuminate\Foundation\Testing\Concerns\CreatesApplication;

abstract class TestCase extends BaseTestCase
{
    use CreatesApplication;
    // use LazilyRefreshDatabase;
}
