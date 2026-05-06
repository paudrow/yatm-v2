# GitHub Test Case Metrics Report

## Overall Metrics
----------------------------------
- **Total Issues**: {{ total_issues }}
- **Open**: {{ open_count }} ({{ open_pct }}%)
- **Closed Completed**: {{ completed_count }} ({{ completed_pct }}%)
- **Closed Won't Fix**: {{ wont_fix_count }} ({{ wont_fix_pct }}%)
- **Closed Duplicate**: {{ duplicate_count }} ({{ duplicate_pct }}%)

{% if !permutations.is_empty() -%}
## Progress by Permutation Key/Value
----------------------------------
{% for perm in permutations -%}
### {{ perm.key }}
{% for val in perm.values -%}
- **{{ val.value }}**: {{ val.closed_count }}/{{ val.total_count }} closed ({{ val.closed_pct }}%) -- Open: {{ val.open_count }} ({{ val.open_pct }}%), Closed Completed: {{ val.completed_count }} ({{ val.completed_pct }}%), Won't Fix: {{ val.wont_fix_count }} ({{ val.wont_fix_pct }}%), Duplicate: {{ val.duplicate_count }} ({{ val.duplicate_pct }}%)
  <div style="width: {{ val.bar_width_pct }}%; background-color: #cbd5e1; border-radius: 4px; overflow: hidden; display: flex; border: 1px solid #cbd5e1; margin: 4px 0 12px 0;">
    {% if val.has_completed -%}
    <div style="width: {{ val.completed_pct }}%; background-color: #2da44e; height: 14px; display: inline-block;" title="Closed Completed: {{ val.completed_count }} cases ({{ val.completed_pct }}%)"></div>
    {%- endif %}
    {% if val.has_open -%}
    <div style="width: {{ val.open_pct }}%; background-color: #ffffff; border-left: 1px solid #ddd; border-right: 1px solid #ddd; height: 14px; display: inline-block;" title="Open: {{ val.open_count }} cases ({{ val.open_pct }}%)"></div>
    {%- endif %}
    {% if val.has_wont_fix -%}
    <div style="width: {{ val.wont_fix_pct }}%; background-color: darkgray; height: 14px; display: inline-block;" title="Won't Fix: {{ val.wont_fix_count }} cases ({{ val.wont_fix_pct }}%)"></div>
    {%- endif %}
    {% if val.has_duplicate -%}
    <div style="width: {{ val.duplicate_pct }}%; background-color: #7c3aed; height: 14px; display: inline-block;" title="Duplicate: {{ val.duplicate_count }} cases ({{ val.duplicate_pct }}%)"></div>
    {%- endif %}
  </div>
{% endfor %}
{% endfor %}
{%- endif %}

{% if !pairwise_matrices.is_empty() -%}
## Pairwise Permutation Progress Matrices
----------------------------------
{% for matrix in pairwise_matrices -%}
### {{ matrix.key_a }} vs {{ matrix.key_b }}

<table style="width: 100%; border-collapse: collapse;">
  <thead>
    <tr>
      <th style="border: 1px solid #ddd; padding: 8px; background-color: #f8fafc;">{{ matrix.key_a }} \ {{ matrix.key_b }}</th>
      {%- for header in matrix.headers %}
      <th style="width: {{ header.width_pct }}%; border: 1px solid #ddd; padding: 8px; background-color: #f8fafc;">{{ header.value }}</th>
      {%- endfor %}
    </tr>
  </thead>
  <tbody>
    {%- for row in matrix.rows %}
    <tr style="height: {{ row.height }}px;">
      <td style="border: 1px solid #ddd; padding: 8px; font-weight: bold; background-color: #f8fafc;">{{ row.val_a }}</td>
      {%- for cell in row.cells %}
      <td style="background-color: hsl(140, 70%, {{ cell.lightness }}%); color: #111; font-weight: 500; border: 1px solid #ddd; padding: 8px; text-align: center; vertical-align: middle;">{%- if cell.has_mini_bar -%}<div style="width: 100%; min-width: 80px; background-color: #cbd5e1; border-radius: 2px; overflow: hidden; display: flex; border: 1px solid #cbd5e1; margin: 2px 0 4px 0;">{%- if cell.has_completed -%}<div style="width: {{ cell.completed_pct }}%; background-color: #2da44e; height: 10px;" title="Closed Completed: {{ cell.completed }} cases ({{ cell.completed_pct }}%)"></div>{%- endif -%}{%- if cell.has_open -%}<div style="width: {{ cell.open_pct }}%; background-color: #ffffff; border-left: 1px solid #ddd; border-right: 1px solid #ddd; height: 10px;" title="Open: {{ cell.open }} cases ({{ cell.open_pct }}%)"></div>{%- endif -%}{%- if cell.has_wont_fix -%}<div style="width: {{ cell.wont_fix_pct }}%; background-color: darkgray; height: 10px;" title="Won't Fix: {{ cell.wont_fix }} cases ({{ cell.wont_fix_pct }}%)"></div>{%- endif -%}{%- if cell.has_duplicate -%}<div style="width: {{ cell.duplicate_pct }}%; background-color: #7c3aed; height: 10px;" title="Duplicate: {{ cell.duplicate }} cases ({{ cell.duplicate_pct }}%)"></div>{%- endif -%}</div>{%- endif -%}Cases: {{ cell.total_cases }}<br>Completed: {{ cell.completed }}/{{ cell.total_valid }} ({{ cell.completed_pct }}%)</td>
      {%- endfor %}
    </tr>
    {%- endfor %}
  </tbody>
</table>

{% endfor %}
{%- endif %}
