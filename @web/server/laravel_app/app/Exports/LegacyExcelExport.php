<?php

namespace App\Exports;

use Illuminate\Support\Collection;
use Illuminate\Support\Str;

class LegacyExcelExport {

    protected $data;
    protected $columns;

    /**
     * Constructor that takes a Collection of data
     *
     * @param Collection $data
     * @param array|null $columns Custom column names (optional)
     */
    public function __construct(Collection $data, array $columns = null)
    {
        $this->data = $data;

        // If no custom columns are provided, use the keys from the first item
        if ($columns === null && $data->isNotEmpty()) {
            $this->columns = array_keys($data->first()->toArray());
        } else {
            $this->columns = $columns ?? [];
        }
    }

    /**
     * Get the sheet content as an array of rows
     *
     * @return array
     */
    public function getSheetContent()
    {
        $rows = [];

        // Add headers as first row
        $rows[] = $this->columns;

        // Add data rows
        foreach ($this->data as $item) {
            $row = [];
            foreach ($this->columns as $column) {
                // Handle nested JSON fields
                if (Str::contains($column, '.')) {
                    $value = data_get($item, $column);
                } else {
                    $value = $item->{$column} ?? null;
                }

                // Handle JSON fields
                if (is_array($value) || is_object($value)) {
                    $value = json_encode($value);
                }

                $row[] = $value;
            }
            $rows[] = $row;
        }

        return $rows;
    }
}
