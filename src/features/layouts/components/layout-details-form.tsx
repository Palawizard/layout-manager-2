import { useEffect } from "react";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";

import { Input } from "../../../components/ui/input";
import { Label } from "../../../components/ui/label";
import { layoutDetailsSchema, type LayoutDetailsValues } from "../schemas/layout-schema";

interface LayoutDetailsFormProps {
  defaultValues: LayoutDetailsValues;
  onChange: (values: LayoutDetailsValues) => void;
}

export function LayoutDetailsForm({ defaultValues, onChange }: LayoutDetailsFormProps) {
  const {
    formState: { errors },
    register,
    watch,
  } = useForm<LayoutDetailsValues>({
    defaultValues,
    resolver: zodResolver(layoutDetailsSchema),
    mode: "onChange",
  });

  useEffect(() => {
    const subscription = watch((values) => {
      if (values.name === undefined) return;
      onChange({
        name: values.name,
        description: values.description ?? "",
      });
    });
    return () => subscription.unsubscribe();
  }, [onChange, watch]);

  return (
    <div className="grid gap-4">
      <div>
        <Label htmlFor="layout-name">Nom</Label>
        <Input id="layout-name" {...register("name")} className="mt-2" />
        {errors.name ? <p className="mt-2 text-sm text-danger">{errors.name.message}</p> : null}
      </div>
      <div>
        <Label htmlFor="layout-description">Description</Label>
        <textarea
          className="mt-2 min-h-24 w-full rounded-md border border-border bg-surface px-3 py-2 text-sm"
          id="layout-description"
          {...register("description")}
        />
        {errors.description ? (
          <p className="mt-2 text-sm text-danger">{errors.description.message}</p>
        ) : null}
      </div>
    </div>
  );
}
