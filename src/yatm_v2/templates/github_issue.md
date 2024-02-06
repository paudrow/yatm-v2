{{ description }}

{% if selected_permutation.len() > 0 -%}
## Configuration

{% for (key, value) in selected_permutation -%}
* {{ key }}: {{ value }}
{% endfor %}
{%- endif %}

{% if links.len() > 0 -%}
## Links

{% for link in links -%}
* [{{ link.name }}]({{ link.url }})
{% endfor %}
{%- endif %}

## Process
{% for step in steps %}
### Step {{ loop.index }}

#### Actions
    {% for action in step.action -%}
        {%- match action -%}
            {% when Action::StdIn with (terminal) %}
```bash
# StdIn - terminal {{ terminal.number }}
{{ terminal.text }}
```
            {% when Action::Image with (image_path) %}
![Image]({{ image_path }})
            {% when Action::Describe with (description) %}
{{ description }}
            {% when Action::Url with (link) %}
[{{ link.name }}]({{ link.url }})
        {%- endmatch -%}
    {%- endfor %}

#### Expected Result
    {% for expect in step.expect -%}
        {%- match expect -%}
            {% when Expect::StdOut with (terminal) %}
```bash
# StdOut - terminal {{ terminal.number }}
{{ terminal.text }}
```
            {% when Expect::StdErr with (terminal) %}
```bash
# StdErr - terminal {{ terminal.number }}
{{ terminal.text }}
```
            {% when Expect::Image with (image_path) %}
![Image]({{ image_path }})
            {% when Expect::Describe with (description) %}
{{ description }}
            {% when Expect::Url with (link) %}
[{{ link.name }}]({{ link.url }})
        {%- endmatch -%}
    {%- endfor -%}
{%- endfor %}
