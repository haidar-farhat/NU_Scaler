<?php

namespace App\Exports;

use Illuminate\Support\Collection;
use Illuminate\Support\Str;
use Maatwebsite\Excel\Concerns\FromArray;
use Maatwebsite\Excel\Concerns\WithHeadings;
use Maatwebsite\Excel\Concerns\ShouldAutoSize;

class LegacyExcelExport implements FromArray, WithHeadings, ShouldAutoSize
{
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
     * @return array
     */
    public function array(): array
    {
        $rows = [];

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

    /**
     * @return array
     */
    public function headings(): array
    {
        return $this->columns;
    }

    /**
     * Get the sheet content as an array of rows (for backward compatibility)
     *
     * @return array
     */
    public function getSheetContent()
    {
        return array_merge([$this->headings()], $this->array());
    }
}
