{{ description }}

{% if selected_permutation.len() > minimum_permutations_to_render -%}
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
    {% match step.name %}
        {% when Some with (name) %}
            {% if steps.len() > 1 %}
### Step {{ loop.index }} - {{name}}
            {% else %}
### {{name}}
            {% endif %}
        {% when None %}
### Step {{ loop.index }}
    {% endmatch %}

    {% match step.description %}
        {% when Some with (description) %}
{{description}}
        {% when None %}
    {% endmatch %}

    {% if step.action.len() > 0 %}
        {% if step.expect.len() > 0 %}
            {% if step.action.len() == 1 %}
#### Action
            {% else %}
#### Actions
            {% endif %}
        {% endif %}
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
    {% endif %}

    {% if step.expect.len() > 0 %}
        {% if step.expect.len() == 1 %}
#### Expected Result
        {% else %}
#### Expected Results
        {% endif %}
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
    {% endif %}
{%- endfor %}
