{{ description }}

{% if selected_permutation.len() > 1 -%}
## Configuration

{% for (key, value) in selected_permutation -%}
* {{ key }}: {{ value }}
{% endfor %}
{%- endif %}

## Process
{% for step in steps %}
### Step {{ loop.index }}

#### Actions
    {% for action in step.action -%}
        {%- match action -%}
            {% when Action::StdIn with (terminal) %}
1. StdIn: `{{ terminal.text }}`
            {% when Action::Image with (image_path) %}
1. ![Image]({{ image_path }})
            {% when Action::Describe with (description) %}
1. Description: {{ description }}
        {%- endmatch -%}
    {%- endfor %}

#### Expected Result
    {% for expect in step.expect -%}
        {%- match expect -%}
            {% when Expect::StdOut with (terminal) %}
1. StdOut: {{ terminal.number }} - {{ terminal.text }}
            {% when Expect::StdErr with (terminal) %}
1. StdErr: {{ terminal.number }} - {{ terminal.text }}
            {% when Expect::Image with (image) %}
1. Image: {{ image }}
            {% when Expect::Describe with (description) %}
1. Description: {{ description }}
        {%- endmatch -%}
    {%- endfor -%}
{%- endfor %}
